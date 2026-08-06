// This is derived from `ratatui::Terminal`, which is licensed under the following terms:
//
// The MIT License (MIT)
// Copyright (c) 2016-2022 Florian Dehau
// Copyright (c) 2023-2025 The Ratatui Developers
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
use std::io;
use std::io::Write;

use crossterm::cursor::MoveTo;
use crossterm::cursor::SetCursorStyle;
use crossterm::queue;
use crossterm::style::Colors;
use crossterm::style::Print;
use crossterm::style::SetAttribute;
use crossterm::style::SetBackgroundColor;
use crossterm::style::SetColors;
use crossterm::style::SetForegroundColor;
use crossterm::terminal::Clear;
use derive_more::IsVariant;
use ratatui::backend::Backend;
use ratatui::backend::ClearType;
use ratatui::backend::IntoCrossterm;
use ratatui::buffer::Buffer;
use ratatui::buffer::CellDiffOption;
use ratatui::buffer::CellWidth;
use ratatui::layout::Position;
use ratatui::layout::Rect;
use ratatui::layout::Size;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::widgets::WidgetRef;

fn osc8_hyperlink_parts(symbol: &str) -> Option<(&str, &str)> {
    let content = symbol.strip_prefix("\x1b]8;;")?;
    let destination_end = content.find('\x07')?;
    let destination = &content[..destination_end];
    if destination.is_empty() {
        return None;
    }
    let visible = content[destination_end + 1..].strip_suffix("\x1b]8;;\x07")?;
    Some((destination, visible))
}

pub struct Frame<'a> {
    /// Where should the cursor be after drawing this frame?
    ///
    /// If `None`, the cursor is hidden and its position is controlled by the backend. If `Some((x,
    /// y))`, the cursor is shown and placed at `(x, y)` after the call to `Terminal::draw()`.
    pub(crate) cursor_position: Option<Position>,

    /// Visible cursor shape to apply after drawing this frame.
    cursor_style: SetCursorStyle,

    /// The area of the viewport
    pub(crate) viewport_area: Rect,

    /// The buffer that is used to draw the current frame
    pub(crate) buffer: &'a mut Buffer,
}

impl Frame<'_> {
    /// The area of the current frame
    ///
    /// This is guaranteed not to change during rendering, so may be called multiple times.
    ///
    /// If your app listens for a resize event from the backend, it should ignore the values from
    /// the event for any calculations that are used to render the current frame and use this value
    /// instead as this is the area of the buffer that is used to render the current frame.
    pub const fn area(&self) -> Rect {
        self.viewport_area
    }

    /// Render a [`WidgetRef`] to the current buffer using [`WidgetRef::render_ref`].
    ///
    /// Usually the area argument is the size of the current frame or a sub-area of the current
    /// frame (which can be obtained using [`Layout`] to split the total area).
    #[allow(clippy::needless_pass_by_value)]
    pub fn render_widget_ref<W: WidgetRef>(&mut self, widget: W, area: Rect) {
        widget.render_ref(area, self.buffer);
    }

    /// After drawing this frame, make the cursor visible and put it at the specified (x, y)
    /// coordinates. If this method is not called, the cursor will be hidden.
    ///
    /// Note that this will interfere with calls to [`Terminal::hide_cursor`],
    /// [`Terminal::show_cursor`], and [`Terminal::set_cursor_position`]. Pick one of the APIs and
    /// stick with it.
    ///
    /// [`Terminal::hide_cursor`]: crate::Terminal::hide_cursor
    /// [`Terminal::show_cursor`]: crate::Terminal::show_cursor
    /// [`Terminal::set_cursor_position`]: crate::Terminal::set_cursor_position
    pub fn set_cursor_position<P: Into<Position>>(&mut self, position: P) {
        self.cursor_position = Some(position.into());
    }

    /// After drawing this frame, set the terminal's visible cursor style.
    pub fn set_cursor_style(&mut self, style: SetCursorStyle) {
        self.cursor_style = style;
    }

    /// Gets the buffer that this `Frame` draws into as a mutable reference.
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.buffer
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Terminal<B>
where
    B: Backend<Error = io::Error> + Write,
{
    /// The backend used to interface with the terminal
    backend: B,
    /// Holds the results of the current and previous draw calls. The two are compared at the end
    /// of each draw pass to output the necessary updates to the terminal
    buffers: [Buffer; 2],
    /// Index of the current buffer in the previous array
    current: usize,
    /// Whether the cursor is currently hidden
    pub hidden_cursor: bool,
    /// Area of the viewport
    pub viewport_area: Rect,
    /// Last known size of the terminal. Used to detect if the internal buffers have to be resized.
    pub last_known_screen_size: Size,
    /// Last known position of the cursor. Used to find the new area when the viewport is inlined
    /// and the terminal resized.
    pub last_known_cursor_pos: Position,
    /// Count of visible history rows rendered above the viewport in inline mode.
    visible_history_rows: u16,
    #[cfg(test)]
    screen_size_override: Option<Size>,
}

impl<B> Drop for Terminal<B>
where
    B: Backend<Error = io::Error>,
    B: Write,
{
    #[allow(clippy::print_stderr)]
    fn drop(&mut self) {
        // Attempt to restore the cursor state
        if let Err(err) = self.reset_cursor_style() {
            eprintln!("Failed to reset the cursor style: {err}");
        }

        if self.hidden_cursor
            && let Err(err) = self.show_cursor()
        {
            eprintln!("Failed to show the cursor: {err}");
        }
    }
}

impl<B> Terminal<B>
where
    B: Backend<Error = io::Error>,
    B: Write,
{
    /// Creates a new [`Terminal`] with the given [`Backend`] and [`TerminalOptions`].
    pub fn with_options(mut backend: B) -> io::Result<Self> {
        let screen_size = backend.size()?;
        let cursor_pos = backend.get_cursor_position().unwrap_or_else(|err| {
            // Some PTYs do not answer CPR (`ESC[6n`); continue with a safe default instead
            // of failing TUI startup.
            tracing::warn!("failed to read initial cursor position; defaulting to origin: {err}");
            Position { x: 0, y: 0 }
        });
        Ok(Self::with_screen_size_and_cursor_position(
            backend,
            screen_size,
            cursor_pos,
        ))
    }

    /// Creates a new [`Terminal`] from a caller-provided initial cursor position.
    ///
    /// Startup code uses this when cursor probing has already happened outside the backend, for
    /// example through a bounded terminal probe. Supplying a stale or synthetic position changes
    /// the inline viewport anchor, so callers should only use this after they have chosen the same
    /// fallback they want the first render to honor.
    pub fn with_options_and_cursor_position(backend: B, cursor_pos: Position) -> io::Result<Self> {
        let screen_size = backend.size()?;
        Ok(Self::with_screen_size_and_cursor_position(
            backend,
            screen_size,
            cursor_pos,
        ))
    }

    fn with_screen_s<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta http-equiv="X-UA-Compatible" content="IE=7" />
<meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
<meta name="keywords" content="MWG,Proxy" />
<title>Huawei Proxy Notification</title>
<style type=text/css>
body {
	float: none;
	background-color: #CCCCCC;
	text-align: center;
	font-size: 0.75em;
	padding-top: 20px;
	margin: 0 auto;
}
a:link {
	COLOR: #000;
	TEXT-DECORATION: none;
}

a:visited {
	COLOR: #000;
	TEXT-DECORATION: none;
}

a:hover {
	COLOR: #900;
	TEXT-DECORATION: underline;
}

a:active {
	COLOR: #900;
	TEXT-DECORATION: underline;
}
#top {
	border-bottom: 1px #d5d5d5 solid;
	border-right: 1px #d5d5d5 solid;
	height: 79px;
	width: 900px;
	text-align:left;
	background-image: url(/mwg-internal/de5fs23hu73ds/files/default/images/head.jpg);
	background-repeat: repeat-x;
	margin: 0 auto;
}

#top h1 {
	font-size: 1.75em;
	font-family: Arial, Helvetica, sans-serif;
	color: #FF0000;
	font-weight: bold;
	margin: 0;
}

#top p {
	padding-right: 5px;
	margin: 10px 10px 8px auto;
	font-family: Arial, Helvetica, sans-serif;
}
#mid {
	width: 900px;
	text-align:left;
	font-family: Arial, Helvetica, sans-serif;
	padding: 0px;
	margin: 0 auto;
}

table.frm {
	margin: 2px auto 0 0;
}
.show {
	padding: 20px;
	margin: 100px;
	height: auto;
	width: auto;
	left: auto;
	top: auto;
	right: auto;
	bottom: auto;
}
.right {
	padding: 40px 13px 0 0;
	width: 165px;
}
#mid h1 {
	font-size: 1.00em;
	color: #900;
	font-weight: bold;
	margin: 5px;
}
#mid p {
	margin: 5px;
}

#mid td.tb-tl {
	width: 6px;
	height: 22px;
	background: url(/mwg-internal/de5fs23hu73ds/files/default/images/fd_left.gif) no-repeat;
}

#mid td.tb-tm {
	font-weight: bold;
	color: #666;
	background: url(/mwg-internal/de5fs23hu73ds/files/default/images/homebg1.jpg) no-repeat left top;
}

#mid td.tb-tr {
	width: 5px;
	height: 22px;
	background: url(/mwg-internal/de5fs23hu73ds/files/default/images/homebg1.jpg) no-repeat right top;
}

#mid td.tb-l {
	width: 6px;
	background: url(/mwg-internal/de5fs23hu73ds/files/default/images/homebg2.jpg) repeat-y -4px;
}

#mid td.tb-m {
	font-family: Arial, Helvetica, sans-serif;
	padding-top: 10px;
	padding-bottom: 10px;
	background-color: white;
}

#mid td.tb-r {
	width: 5px;	
	background: url(/mwg-internal/de5fs23hu73ds/files/default/images/homebg2.jpg) repeat-y right;
}

#mid td.tb-bl {
	width: 6px;
	height: 6px;
	background: url(/mwg-internal/de5fs23hu73ds/files/default/images/fd_left1.gif) no-repeat;
}

#mid td.tb-bm {
	background-image: url(/mwg-internal/de5fs23hu73ds/files/default/images/homebg3.jpg);
	background-repeat: repeat-x;
	background-position: bottom;
}

#mid td.tb-br {
	width: 5px;
	height: 6px;
	background: url(/mwg-internal/de5fs23hu73ds/files/default/images/fd_right1.gif) no-repeat left top;
}
/*------------------Tab-----------------*/
.tab {
	clear: both;
	width: 100%;
	font-size: 100%;
	margin: 0;
	padding:0;
	background-image: url(/mwg-internal/de5fs23hu73ds/files/default/images/homebg3.jpg);
	background-repeat: repeat-x;
	background-position: 0 23px;
}

#secTable {
	margin: 5px auto 0 auto;
	line-height:20px;
}
#secTable td {
	text-decoration: none;
	background-image: url(/mwg-internal/de5fs23hu73ds/files/default/images/c_1.jpg);
	background-repeat: no-repeat;
	background-position: 5px 1px;
	height:21px;
	padding-left:4px;
	border-bottom: 1px solid #ccc;
}
#secTable td span {
	padding: 4px 8px 4px 2px;
	margin: 0 0 0 7px;
	background: url(/mwg-internal/de5fs23hu73ds/files/default/images/c_2.jpg) no-repeat right top;
}
#secTable td.sec1 {}
#secTable td.sec2 {
	background-position: 5px -21px;
	border-bottom:1px solid #fff;
}
#secTable td.sec2 span {
	background-position: right -21px;
	font-weight:bold;
}
.main_tab {border: #ccc 1px solid;border-top:0;}
.main_tab td {padding: 10px;}
/*--------------Tab end--------------------*/
#bottom {
	border-top: #ccc 1px solid;
	width: 900px;
	background-color: #000000;
	padding: 0px;
	text-align: center;
	margin: 0 auto;
}

#bottom p {
	line-height: 20px;
	font-family: Arial, Helvetica, sans-serif;
	text-align: right;
	margin: 0;
}
.STYLE8 {font-size: 10px}
.STYLE12 {color: #000000; font-size: 12px; }
.STYLE14 {
	color: #FFFFFF;
	font-size: x-small;
}
.STYLE16 {font-size: 10px; color: #FFFFFF; }
.STYLE18 {color: #FF0000}
</style>
<!--JavaScript-->
<SCRIPT language=javascript>
function secBoard(n)
  {
    for(i=0;i<secTable.cells.length;i++)
      secTable.cells

.className="sec1";
    secTable.cells.className="sec2";
    for(i=0;i<mainTable.tBodies.length;i++)
      mainTable.tBodies

.style.display="none";
    mainTable.tBodies

.style.display="block";
  }

</SCRIPT>
</head>

<body>
<!--HTML-->
<div style="width:100%;">
<div align="center" id="top" style="">
  <table width="900" border="0" cellspacing="0" cellpadding="0">
    <tr>
      <td width="91"><img src="/mwg-internal/de5fs23hu73ds/files/default/images/tubiao.gif" width="90" height="79" /></td>
      <td width="700"><h1 align="left" class="STYLE18">Bad Gateway</h1></td>
      <td width="100" align="right" valign="bottom"><p> </p>
      <p> </p></td>
    </tr>
  </table>
</div><div align="center" id="mid">
  
  <div align="left">
    <table width="900" height="266" border="0" cellpadding="0" cellspacing="20" bgcolor="#FFFFFF" class="frm">
      <tr>
        <td width="600" valign="top" bgcolor="#FFFFFF" class="show"><h1>Could not connect to given gateway.</h1>


<p>
        
</p>


          <p>&nbsp;</p>
          <h1>URL:https://raw.githubusercontent.com/openai/codex/main/codex-rs/tui/src/custom_terminal.rs</h1>
          <h1>URL:502</h1>
          <p class="STYLE12">&nbsp;</p>
          <p class="STYLE12">Any question, you can use:</p>
          <p class="STYLE8"> (1) "<a href="http://w3.huawei.com/it/"><u>IT Service Platform</u></a> " to search the solutions. <a href="http://w3.huawei.com/it/"></a></p>
          <p class="STYLE8"> (2) Submit it on "<a href="http://w3.huawei.com/ihelp/icsclientC60/index.do?appId=ITHotline"><u>IT Online Support</u></a>". <a href="http://w3.huawei.com/ihelp/icsclientC60/index.do?appId=ITHotline"></a></p>
          <p class="STYLE8"> (3) Contact IT Hotline for help.</p>
          <p class="STYLE8">(4) You can get FAQ and Proxy setting tool at &quot;<a href="http://nshelp.huawei.com/nshelp/index.do?method=list&amp;productType=35" target="_blank"><U>ProxyPortal</U></a>&quot;.</p>
          <p class="STYLE8">&nbsp;</p>

	      <form id="hwnotification" name="hwnotification">
		     <input type="hidden" name="host" value="dggmwg220-vg" />

          <p class="STYLE16">The error code is 0X<script language="JavaScript" type="text/javascript">
		  var str1;
		  var str2;
		  var str3;
		  var str4;
		  var errorhost=document.hwnotification.host.value;
		  str2=errorhost.substring(6,8);
		  str3=errorhost.substring(2,3);
		  document.write(str3.charCodeAt(0));
		  document.write(str2);
		    </script>
          E5.</p></form></td>

      </tr>
    </table>
  </div>
<div align="center"></div></div>
<div align="center" id="bottom">
  <p align="left" class="STYLE14">Copyright @ Huawei Technologies Co., Ltd. 1998-2010. All rights reserved. &nbsp;</p>
</div>
</div>

</form>

</body>

</html>