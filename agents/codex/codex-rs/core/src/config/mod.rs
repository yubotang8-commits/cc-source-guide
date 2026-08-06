use crate::config::edit::ConfigEdit;
use crate::config::edit::ConfigEditsBuilder;
use crate::path_utils::normalize_for_native_workdir;
use crate::unified_exec::DEFAULT_MAX_BACKGROUND_TERMINAL_TIMEOUT_MS;
use crate::unified_exec::MIN_EMPTY_YIELD_TIME_MS;
use crate::windows_sandbox::WindowsSandboxLevelExt;
use crate::windows_sandbox::resolve_windows_sandbox_mode;
use crate::windows_sandbox::resolve_windows_sandbox_private_desktop;
use codex_config::CloudConfigBundleLoader;
use codex_config::ConfigLayerSource;
use codex_config::ConfigLayerStack;
use codex_config::ConfigRequirements;
use codex_config::ConfigRequirementsToml;
use codex_config::ConstrainedWithSource;
use codex_config::FeatureRequirementsToml;
use codex_config::McpServerRequirement;
use codex_config::PluginRequirementsToml;
use codex_config::ProfileV2Name;
use codex_config::ResidencyRequirement;
use codex_config::SandboxModeRequirement;
use codex_config::Sourced;
use codex_config::ThreadConfigLoader;
use codex_config::config_toml::ConfigLockfileToml;
use codex_config::config_toml::ConfigToml;
use codex_config::config_toml::DEFAULT_PROJECT_DOC_MAX_BYTES;
use codex_config::config_toml::ProjectConfig;
use codex_config::config_toml::RealtimeAudioConfig;
use codex_config::config_toml::RealtimeConfig;
use codex_config::config_toml::ThreadStoreToml;
use codex_config::config_toml::validate_model_providers;
use codex_config::loader::load_config_layers_state;
use codex_config::loader::project_trust_key;
use codex_config::permissions_toml::PermissionsToml;
use codex_config::sandbox_mode_requirement_for_permission_profile;
use codex_config::types::ApprovalsReviewer;
use codex_config::types::AuthCredentialsStoreMode;
use codex_config::types::AuthKeyringBackendKind;
use codex_config::types::History;
use codex_config::types::McpServerConfig;
use codex_config::types::McpServerDisabledReason;
use codex_config::types::MemoriesConfig;
use codex_config::types::ModelAvailabilityNuxConfig;
use codex_config::types::Notice;
use codex_config::types::OAuthCredentialsStoreMode;
use codex_config::types::ResumeCwdMode;
use codex_config::types::SessionPickerViewMode;
use codex_config::types::ToolSuggestConfig;
use codex_config::types::ToolSuggestDisabledTool;
use codex_config::types::ToolSuggestDiscoverable;
use codex_config::types::TuiKeymap;
use codex_config::types::TuiNotificationSettings;
use codex_config::types::TuiPetAnchor;
use codex_config::types::UriBasedFileOpener;
use codex_config::types::WindowsSandboxModeToml;
use codex_core_plugins::PluginLoadOutcome;
use codex_core_plugins::PluginsConfigInput;
use codex_exec_server::ExecutorFileSystem;
use codex_exec_server::LOCAL_FS;
use codex_features::CodeModeConfigToml;
use codex_features::CurrentTimeReminderConfigToml;
use codex_features::CurrentTimeReminderDeliveryMode;
use codex_features::CurrentTimeSource;
use codex_features::Feature;
use codex_features::FeatureConfigSource;
use codex_features::FeatureOverrides;
use codex_features::FeatureToml;
use codex_features::Features;
use codex_features::FeaturesToml;
use codex_features::MultiAgentV2ConfigToml;
use codex_features::NetworkProxyConfigToml;
use codex_features::TokenBudgetConfigToml;
use codex_git_utils::resolve_root_git_project_for_trust;
use codex_http_client::HttpClientFactory;
use codex_http_client::OutboundProxyPolicy;
use codex_install_context::InstallContext;
use codex_login::AuthManagerConfig;
use codex_login::AuthRouteConfig;
use codex_mcp::McpConfig;
use codex_mcp::McpPluginAttribution;
use codex_mcp::McpProtocolMode;
use codex_mcp::McpServerRegistration;
use codex_mcp::ResolvedMcpCatalog;
use codex_memories_read::memory_root;
use codex_model_provider_info::LEGACY_OLLAMA_CHAT_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::OLLAMA_CHAT_PROVIDER_REMOVED_ERROR;
use codex_model_provider_info::built_in_model_providers;
use codex_model_provider_info::merge_configured_model_providers;
use codex_models_manager::ModelsManagerConfig;
use codex_protocol::config_types::AltScreenMode;
use codex_protocol::config_types::AutoCompactTokenLimitScope;
use codex_protocol::config_types::ForcedLoginMethod;
use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE;
use codex_protocol::config_types::SandboxMode;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::config_types::TrustLevel;
use codex_protocol::config_types::Verbosity;
use codex_protocol::config_types::WebSearchConfig;
use codex_protocol::config_types::WebSearchMode;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::models::ActivePermissionProfile;
use codex_protocol::models::PermissionProfile;
use codex_protocol::models::SandboxEnforcement;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::SandboxPolicy;
pub use codex_thread_store::ExtraConfig;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_absolute_path::AbsolutePathBufGuard;
use codex_utils_path_uri::PathUri;
use rmcp::model::ElicitationCapability;
use rmcp::model::FormElicitationCapability;
use rmcp::model::UrlElicitationCapability;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::permissions::BUILT_IN_READ_ONLY_PROFILE;
use crate::config::permissions::BUILT_IN_WORKSPACE_PROFILE;
use crate::config::permissions::apply_network_proxy_feature_config;
use crate::config::permissions::builtin_permission_profile;
use crate::config::permissions::compile_permission_profile_selection;
use crate::config::permissions::compile_permission_profile_workspace_roots;
use crate::config::permissions::default_builtin_permission_profile_name;
use crate::config::permissions::get_readable_roots_required_for_codex_runtime;
use crate::config::permissions::network_proxy_config_for_profile_selection;
use crate::config::permissions::validate_user_permission_profile_names;
use crate::config_lock::config_without_lock_controls;
use crate::config_lock::lock_layer_from_config;
use crate::config_lock::read_config_lock_from_path;
use codex_network_proxy::NetworkProxyConfig;
use toml::Value as TomlValue;
use toml_edit::DocumentMut;

pub(crate) mod agent_roles;
mod auth_keyring;
pub mod edit;
mod managed_features;
mod network_proxy_spec;
mod otel;
mod permission_profile_catalog;
mod permissions;
mod requirements;
mod resolved_permission_profile;
#[cfg(test)]
mod schema;
pub use auth_keyring::resolve_bootstrap_auth_keyring_backend_kind;
pub use codex_config::ConfigLoadOptions;
pub use codex_config::Constrained;
pub use codex_config::ConstraintError;
pub use codex_config::ConstraintResult;
pub use codex_config::LoaderOverrides;
pub use codex_network_proxy::NetworkProxyAuditMetadata;
use codex_sandboxing::compatibility_sandbox_policy_for_permission_profile;
pub use codex_sandboxing::system_bwrap_warning;
pub use managed_features::ManagedFeatures;
pub use network_proxy_spec::NetworkProxySpec;
pub use network_proxy_spec::StartedNetworkProxy;
pub use permission_profile_catalog::PermissionProfileCatalogEntry;
pub use permission_profile_catalog::permission_profile_catalog;
use permission_profile_catalog::permission_profile_catalog_from_permissions;
use permission_profile_catalog::permission_profile_is_allowed;
use permission_profile_catalog::validate_permission_profile_for_deny_read;
pub(crate) use permissions::is_builtin_permission_profile_name;
pub use resolved_permission_profile::PermissionProfileSnapshot;
pub(crate) use resolved_permission_profile::PermissionProfileState;

const DEFAULT_IGNORE_LARGE_UNTRACKED_DIRS: i64 = 200;
const DEFAULT_IGNORE_LARGE_UNTRACKED_FILES: i64 = 10 * 1024 * 1024;

/// Compatibility-only config retained so legacy `ghost_snapshot` settings
/// continue to load even though snapshots are no longer produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhostSnapshotConfig {
    pub ignore_large_untracked_files: Option<i64>,
    pub ignore_large_untracked_dirs: Option<i64>,
    pub disable_warnings: bool,
}

impl Default for GhostSnapshotConfig {
    fn default() -> Self {
        Self {
            ignore_large_untracked_files: Some(DEFAULT_IGNORE_LARGE_UNTRACKED_FILES),
            ignore_large_untracked_dirs: Some(DEFAULT_IGNORE_LARGE_UNTRACKED_DIRS),
            disable_warnings: false,
        }
    }
}

/// Maximum number of bytes of the documentation that will be embedded. Larger
/// files are *silently truncated* to this size so we do not take up too much of
/// the context window.
pub(crate) const AGENTS_MD_MAX_BYTES: usize = DEFAULT_PROJECT_DOC_MAX_BYTES; // 32 KiB
pub(crate) const DEFAULT_AGENT_MAX_THREADS: Option<usize> = Some(6);
pub(crate) const DEFAULT_MULTI_AGENT_V2_MAX_CONCURRENT_THREADS_PER_SESSION: usize = 4;
pub(crate) const DEFAULT_MULTI_AGENT_V2_MIN_WAIT_TIMEOUT_MS: i64 = 10_000;
pub(crate) const DEFAULT_MULTI_AGENT_V2_MAX_WAIT_TIMEOUT_MS: i64 = 3600 * 1000;
pub(crate) const DEFAULT_MULTI_AGENT_V2_DEFAULT_WAIT_TIMEOUT_MS: i64 = 30_000;
const DEFAULT_MULTI_AGENT_V2_ROOT_AGENT_USAGE_HINT_TEXT: &str = r#"You are `/root`, the primary agent in a team of agents collaborating to fulfill the user's goals.

At the start of your turn, you are the active agent.
You can spawn sub-agents to handle subtasks, and those sub-agents can spawn their own sub-agents.
All agents in the team, including the agents that you can assign tasks to, are equally intelligent and capable, and have access to the same set of tools.

You can use `spawn_agent` to create a new agent, `followup_task` to give an existing agent a new task and trigger a turn, and `send_message` to pass a message to a running agent without triggering a turn.
Child agents can also spawn their own sub-agents.
You can decide how much context you want to propagate to your sub-agents with the `fork_turns` parameter.

You will receive messages in the analysis channel in the form:
```
Message Type: MESSAGE | FINAL_ANSWER
Task name: <recipient>
Sender: <author>
Payload:
<payload text>
```
They may be addressed as to=/root
"#;
const DEFAULT_MULTI_AGENT_V2_SUBAGENT_USAGE_HINT_TEXT: &str = r#"You are an agent in a team of agents collaborating to complete a task.

You can spawn sub-agents to handle subtasks, and those sub-agents can spawn their own sub-agents. All agents in the team, including the agents that you can assign tasks to, are equally intelligent and capable, and have access to the same set of tools.

You can use `spawn_agent` to create a new agent, `followup_task` to give an existing agent a new task and trigger a turn, and `send_message` to pass a message to a running agent.
Child agents can also spawn their own sub-agents.

When you provide a response in the final channel, that content is immediately delivered back to your parent agent.

You will receive messages in the analysis channel in the form:
```
Message Type: NEW_TASK | MESSAGE | FINAL_ANSWER
Task name: <recipient>
Sender: <author>
Payload:
<payload text>
```
You may also see them addressed as to=/root/..., which indicates your identity is /root/...
"#;
const DEFAULT_MULTI_AGENT_V2_MODEL_OVERRIDE_USAGE_HINT_TEXT: &str = "Full-history forks (`fork_turns` omitted or `\"all\"`) inherit the parent model and reasoning effort and do not accept overrides. Only set `model` or `reasoning_effort` when explicitly requested by the user, applicable `AGENTS.md` instructions, or skill instructions; when doing so, set `fork_turns` to `\"none\"` or a positive integer string.";
const DEFAULT_MULTI_AGENT_V2_TOOL_NAMESPACE: &str = "collaboration";
const DEFAULT_MULTI_AGENT_V2_SHARED_USAGE_HINT_TEXT: &str = r#"Note that collaboration tools cannot be called from inside `functions.exec`. Call `spawn_agent`, `send_message`, `followup_task`, `wait_agent`, `interrupt_agent`, and `list_agents` only as direct tool calls using the recipient shown in their tool definitions, such as `to=functions.collaboration.spawn_agent`, since they are intentionally absent from the `functions.exec` `tools.*` namespace. Available tools in `functions.exec` are explicitly described with a `tools` namespace in the developer message.

All agents share the same directory. In detail:
- All agents have access to the same container and filesystem as you.
- All agents use the same current working directory.
- As a result, edits made by one agent are immediately visible to all other agents.
"#;
fn default_multi_agent_v2_usage_hint_text(usage_hint_text: &str, max_concurrency: usize) -> String {
    format!(
        "{usage_hint_text}\n{DEFAULT_MULTI_AGENT_V2_SHARED_USAGE_HINT_TEXT}\nThere are {max_concurrency} available concurrency slots, meaning that up to {max_concurrency} agents can be active at once, including you."
    )
}

pub(crate) const HARD_MIN_MULTI_AGENT_V2_TIMEOUT_MS: i64 = 0;
pub(crate) const HARD_MAX_MULTI_AGENT_V2_TIMEOUT_MS: i64 =
    DEFAULT_MULTI_AGENT_V2_MAX_WAIT_TIMEOUT_MS;
pub(crate) const DEFAULT_AGENT_MAX_DEPTH: i32 = 1;
const LOCAL_DEV_BUILD_VERSION: &str = "0.0.0";

pub const CONFIG_TOML_FILE: &str = "config.toml";
const CONFIG_PROFILE_V2_SUFFIX: &str = ".config.toml";

fn resolve_sqlite_home_env(resolved_cwd: &Path) -> Option<AbsolutePathBuf> {
    let raw = std::env::var(codex_state::SQLITE_HOME_ENV).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(AbsolutePathBuf::resolve_path_against_base(
        trimmed,
        resolved_cwd,
    ))
}

fn resolve_cli_auth_credentials_store_mode(
    configured: AuthCredentialsStoreMode,
    package_version: &str,
) -> AuthCredentialsStoreMode {
    match (package_version, configured) {
        (
            LOCAL_DEV_BUILD_VERSION,
            AuthCredentialsStoreMode::Keyring | AuthCredentialsStoreMode::Auto,
        ) => AuthCredentialsStoreMode::File,
        (_, mode) => mode,
    }
}

fn resolve_mcp_oauth_credentials_store_mode(
    configured: OAuthCredentialsStoreMode,
    package_version: &str,
) -> OAuthCredentialsStoreMode {
    match (package_version, configured) {
        (
            LOCAL_DEV_BUILD_VERSION,
            OAuthCredentialsStoreMode::Keyring | OAuthCredentialsStoreMode::Auto,
        ) => OAuthCredentialsStoreMode::File,
        (_, mode) => mode,
    }
}

#[cfg(test)]
pub(crate) async fn test_config() -> Config {
    let codex_home = tempfile::tempdir().expect("create temp dir");
    Config::load_from_base_config_with_overrides(
        ConfigToml {
            model: Some("gpt-5.5".to_string()),
            ..Default::default()
        },
        ConfigOverrides::default(),
        AbsolutePathBuf::from_absolute_path(codex_home.path()).expect("temp dir should resolve"),
    )
    .await
    .expect("load default test config")
}

/// Application configuration loaded from disk and merged with overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct Permissions {
    /// Approval policy for executing commands.
    pub approval_policy: Constrained<AskForApproval>,
    /// Constrained permission profile plus its selected profile identity, if
    /// the profile came from a built-in or named config profile.
    permission_profile_state: PermissionProfileState,
    /// Thread-scoped runtime workspace roots. Symbolic `:workspace_roots`
    /// entries in the permission profile are materialized against these roots.
    workspace_roots: Vec<AbsolutePathBuf>,
    /// Effective network configuration applied to all spawned processes.
    pub network: Option<NetworkProxySpec>,
    /// Whether the model may request a login shell for shell-based tools.
    /// Default to `true`
    ///
    /// If `true`, the model may request a login shell (`login = true`), and
    /// omitting `login` defaults to using a login shell.
    /// If `false`, the model can never use a login shell: `login = true`
    /// requests are rejected, and omitting `login` defaults to a non-login
    /// shell.
    pub allow_login_shell: bool,
    /// Policy used to build process environments for shell/unified exec.
    pub shell_environment_policy: ShellEnvironmentPolicy,
    /// Effective Windows sandbox mode derived from `[windows].sandbox` or
    /// legacy feature keys.
    pub windows_sandbox_mode: Option<WindowsSandboxModeToml>,
    /// Whether the final Windows sandboxed child should run on a private desktop.
    pub windows_sandbox_private_desktop: bool,
}

impl Permissions {
    /// Build permissions from the constrained values required for a minimal
    /// in-process configuration.
    pub fn from_approval_and_profile(
        approval_policy: Constrained<AskForApproval>,
        permission_profile: Constrained<PermissionProfile>,
    ) -> ConstraintResult<Self> {
        Ok(Self {
            approval_policy,
            permission_profile_state: PermissionProfileState::from_constrained_legacy(
                permission_profile,
            )?,
            workspace_roots: Vec::new(),
            network: None,
            allow_login_shell: true,
            shell_environment_policy: ShellEnvironmentPolicy::default(),
            windows_sandbox_mode: None,
            windows_sandbox_private_desktop: true,
        })
    }

    pub(crate) fn permission_profile_state(&self) -> &PermissionProfileState {
        &self.permission_profile_state
    }

    pub(crate) fn set_permission_profile_state(
        &mut self,
        permission_profile_state: PermissionProfileState,
    ) {
        self.permission_profile_state = permission_profile_state;
    }

    /// Apply a permission profile snapshot emitted by core session state.
    ///
    /// This is a trusted-state bridge for consumers of `SessionConfigured`.
    /// Config loading and app-server selection should resolve named profiles
    /// through config instead of constructing a snapshot directly.
    pub fn set_permission_profile_from_session_snapshot(
        &mut self,
        snapshot: PermissionProfileSnapshot,
    ) -> ConstraintResult<()> {
        self.permission_profile_state
            .set_permission_profile_snapshot(snapshot)
    }

    /// Replace the current permission constraints with a trusted session
    /// snapshot. This is only for clients that must mirror core session state
    /// after their local config constraints reject the snapshot.
    pub fn replace_permission_profile_from_session_snapshot(
        &mut self,
        snapshot: PermissionProfileSnapshot,
    ) -> ConstraintResult<()> {
        let permission_profile = Constrained::allow_only(snapshot.permission_profile().clone());
        self.permission_profile_state = PermissionProfileState::from_constrained_resolved(
            permission_profile,
            snapshot.into_resolved_permission_profile(),
        )?;
        Ok(())
    }

    /// Borrow the canonical profile before runtime workspace-root
    /// materialization has been applied.
    pub fn permission_profile(&self) -> &PermissionProfile {
        self.permission_profile_state.permission_profile()
    }

    pub fn can_set_permission_profile(
        &self,
        permission_profile: &PermissionProfile,
    ) -> ConstraintResult<()> {
        self.permission_profile_state
            .can_set_legacy_permission_profile(permission_profile)
    }

    pub fn set_workspace_roots(&mut self, workspace_roots: Vec<AbsolutePathBuf>) {
        self.workspace_roots = workspace_roots;
    }

    pub fn workspace_roots(&self) -> &[AbsolutePathBuf] {
        &self.workspace_roots
    }

    /// Workspace roots that came from user-visible configuration or runtime
    /// selection. Internal Codex-only writable roots are intentionally excluded.
    pub fn user_visible_workspace_roots(&self) -> &[AbsolutePathBuf] {
        &self.workspace_roots
    }

    pub fn profile_workspace_roots(&self) -> &[AbsolutePathBuf] {
        self.permission_profile_state.profile_workspace_roots()
    }

    /// Effective runtime permissions after config requirements and runtime
    /// workspace-root materialization have been applied.
    pub fn effective_permission_profile(&self) -> PermissionProfile {
        self.permission_profile()
            .clone()
            .materialize_project_roots_with_workspace_roots(&self.workspace_roots)
    }

    /// Named profile selected by config, if the current profile has one.
    pub fn active_permission_profile(&self) -> Option<ActivePermissionProfile> {
        self.permission_profile_state.active_permission_profile()
    }

    /// Effective filesystem sandbox policy derived from the canonical profile.
    pub fn file_system_sandbox_policy(&self) -> FileSystemSandboxPolicy {
        self.effective_permission_profile()
            .file_system_sandbox_policy()
    }

    /// Effective network sandbox policy derived from the canonical profile.
    pub fn network_sandbox_policy(&self) -> NetworkSandboxPolicy {
        self.permission_profile().network_sandbox_policy()
    }

    /// Legacy compatibility projection derived from the canonical profile.
    pub fn legacy_sandbox_policy(&self, cwd: &Path) -> SandboxPolicy {
        let permission_profile = self.effective_permission_profile();
        compatibility_sandbox_policy_for_permission_profile(&permission_profile, cwd)
    }

    /// Check whether a legacy sandbox policy can be applied to this permission
    /// set after projecting it into the canonical permission profile.
    pub fn can_set_legacy_sandbox_policy(
        &self,
        sandbox_policy: &SandboxPolicy,
        cwd: &Path,
    ) -> ConstraintResult<()> {
        let file_system_sandbox_policy =
            FileSystemSandboxPolicy::from_legacy_sandbox_policy_for_cwd(sandbox_policy, cwd);
        let network_sandbox_policy = NetworkSandboxPolicy::from(sandbox_policy);
        let permission_profile = PermissionProfile::from_runtime_permissions_with_enforcement(
            SandboxEnforcement::from_legacy_sandbox_policy(sandbox_policy),
            &file_system_sandbox_policy,
            network_sandbox_policy,
        );
        self.permission_profile_state
            .can_set_legacy_permission_profile(&permission_profile)
    }

    /// Set permissions from a legacy sandbox policy and keep every permission
    /// projection in sync.
    pub fn set_legacy_sandbox_policy(
        &mut self,
        sandbox_policy: SandboxPolicy,
        cwd: &Path,
    ) -> ConstraintResult<()> {
        self.can_set_legacy_sandbox_policy(&sandbox_policy, cwd)?;
        let file_system_sandbox_policy =
            FileSystemSandboxPolicy::from_legacy_sandbox_policy_for_cwd(&sandbox_policy, cwd);
        let network_sandbox_policy = NetworkSandboxPolicy::from(&sandbox_policy);
        let permission_profile = PermissionProfile::from_runtime_permissions_with_enforcement(
            SandboxEnforcement::from_legacy_sandbox_policy(&sandbox_policy),
            &file_system_sandbox_policy,
            network_sandbox_policy,
        );
        self.workspace_roots = match &sandbox_policy {
            SandboxPolicy::WorkspaceWrite { writable_roots, .. } => {
                let mut workspace_roots = vec![
                    AbsolutePathBuf::from_absolute_path(cwd)
                        .unwrap_or_else(|_| AbsolutePathBuf::resolve_path_against_base(cwd, "/")),
                ];
                for root in writable_roots {
                    if !workspace_roots.iter().any(|existing| existing == root) {
                        workspace_roots.push(root.clone());
                    }
                }
                workspace_roots
            }
            SandboxPolicy::DangerFullAccess
            | SandboxPolicy::ExternalSandbox { .. }
            | SandboxPolicy::ReadOnly { .. } => vec![
                AbsolutePathBuf::from_absolute_path(cwd)
                    .unwrap_or_else(|_| AbsolutePathBuf::resolve_path_against_base(cwd, "/")),
            ],
        };

        self.permission_profile_state
            .set_legacy_permission_profile(permission_profile)?;
        Ok(())
    }

    /// Set permissions from the canonical profile.
    pub fn set_permission_profile(
        &mut self,
        permission_profile: PermissionProfile,
    ) -> ConstraintResult<()> {
        self.permission_profile_state
            .set_legacy_permission_profile(permission_profile)
    }
}

// A profile override only inherits the selected profile's proxy/allowlist config
// when Codex is still responsible for the network policy. `Disabled` means no
// outer sandbox, so starting the managed proxy would narrow the override.
fn profile_allows_configured_network_proxy(permission_profile: &PermissionProfile) -> bool {
    match permission_profile {
        PermissionProfile::Managed { network, .. } | PermissionProfile::External { network } => {
            network.is_enabled()
        }
        PermissionProfile::Disabled => false,
    }
}

fn build_network_proxy_spec(
    configured_network_proxy_config: NetworkProxyConfig,
    network_requirements: Option<Sourced<codex_config::NetworkConstraints>>,
    permission_profile: &PermissionProfile,
) -> std::io::Result<Option<NetworkProxySpec>> {
    let (network_requirements, network_requirements_source) = match network_requirements {
        Some(Sourced { value, source }) => (Some(value), Some(source)),
        None => (None, None),
    };
    let has_network_requirements = network_requirements.is_some();
    let network = NetworkProxySpec::from_config_and_constraints(
        configured_network_proxy_config,
        network_requirements,
        permission_profile,
    )
    .map_err(|err| {
        if let Some(source) = network_requirements_source.as_ref() {
            std::io::Error::new(
                err.kind(),
                format!("failed to build managed network proxy from {source}: {err}"),
            )
        } else {
            err
        }
    })?;

    Ok(if has_network_requirements {
        Some(network)
    } else {
        network.enabled().then_some(network)
    })
}

/// Configured thread persistence backend.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ThreadStoreConfig {
    /// Persist threads locally using rollout JSONL files and sqlite metadata.
    #[default]
    Local,
    /// In-memory thread store for test and debug configurations.
    InMemory { id: String },
}

/// Application configuration loaded from disk and merged with overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Provenance for how this [`Config`] was derived (merged layers + enforced
    /// requirements).
    pub config_layer_stack: ConfigLayerStack,

    /// Warnings collected during config load that should be shown on startup.
    pub startup_warnings: Vec<String>,

    /// Optional override of model selection.
    pub model: Option<String>,

    /// Effective service tier request id preference for new turns.
    /// `default` means the user explicitly selected standard routing.
    pub service_tier: Option<String>,

    /// Model used specifically for review sessions.
    pub review_model: Option<String>,

    /// Size of the context window for the model, in tokens.
    pub model_context_window: Option<i64>,

    /// Token usage threshold triggering auto-compaction of conversation history.
    pub model_auto_compact_token_limit: Option<i64>,

    /// Controls whether `model_auto_compact_token_limit` applies to the full
    /// active context or only tokens after the carried compaction-window prefix.
    pub model_auto_compact_token_limit_scope: AutoCompactTokenLimitScope,

    /// Key into the model_providers map that specifies which provider to use.
    pub model_provider_id: String,

    /// Info needed to make an API request to the model.
    pub model_provider: ModelProviderInfo,

    /// Optionally specify the personality of the model
    pub personality: Option<Personality>,

    /// Effective permission configuration for shell tool execution.
    pub permissions: Permissions,

    /// Whether config explicitly selected named permissions profiles instead
    /// of the legacy `sandbox_mode` syntax.
    pub explicit_permission_profile_mode: bool,

    /// User-defined permission profiles available from effective config.
    pub custom_permission_profiles: Vec<PermissionProfileCatalogEntry>,

    /// Configures who approval requests are routed to for review once they have
    /// been escalated. This does not disable separate safety checks such as
    /// ARC.
    pub approvals_reviewer: ApprovalsReviewer,

    /// enforce_residency means web traffic cannot be routed outside of a
    /// particular geography. HTTP clients should direct their requests
    /// using backend-specific headers or URLs to enforce this.
    pub enforce_residency: Constrained<Option<ResidencyRequirement>>,

    /// When `true`, `AgentReasoning` events emitted by the backend will be
    /// suppressed from the frontend output. This can reduce visual noise when
    /// users are only interested in the final agent responses.
    pub hide_agent_reasoning: bool,

    /// When set to `true`, `AgentReasoningRawContentEvent` events will be shown in the UI/output.
    /// Defaults to `false`.
    pub show_raw_agent_reasoning: bool,

    /// Base instructions override.
    pub base_instructions: Option<String>,

    /// Developer instructions override injected as a separate message.
    pub developer_instructions: Option<String>,

    /// Guardian-specific policy config override from requirements.toml or config.toml.
    /// This is inserted into the fixed guardian prompt template under the
    /// `# Policy Configuration` section rather than replacing the whole
    /// guardian developer prompt.
    pub guardian_policy_config: Option<String>,

    /// Whether to inject the `<permissions instructions>` developer block.
    pub include_permissions_instructions: bool,

    /// Whether to inject the `<apps_instructions>` developer block.
    pub include_apps_instructions: bool,

    /// Whether to inject the `<collaboration_mode>` developer block.
    pub include_collaboration_mode_instructions: bool,

    /// Whether to inject the `<skills_instructions>` developer block.
    pub include_skill_instructions: bool,

    /// Whether orchestrator-owned skills are exposed to the model.
    pub orchestrator_skills_enabled: bool,

    /// Whether orchestrator-owned MCP tools are exposed to the model.
    pub orchestrator_mcp_enabled: bool,

    /// Whether to inject the `<environment_context>` user block.
    pub include_environment_context: bool,

    /// Compact prompt override.
    pub compact_prompt: Option<String>,

    /// Optional external notifier command. When set, Codex will spawn this
    /// program after each completed *turn* (i.e. when the agent finishes
    /// processing a user submission). The value must be the full command
    /// broken into argv tokens **without** the trailing JSON argument - Codex
    /// appends one extra argument containing a JSON payload describing the
    /// event.
    ///
    /// Example `~/.codex/config.toml` snippet:
    ///
    /// ```toml
    /// notify = ["notify-send", "Codex"]
    /// ```
    ///
    /// which will be invoked as:
    ///
    /// ```shell
    /// notify-send Codex '{"type":"agent-turn-complete","turn-id":"12345"}'
    /// ```
    ///
    /// If unset the feature is disabled.
    pub notify: Option<Vec<String>>,

    /// TUI notification settings, including enabled events, delivery method, and focus condition.
    pub tui_notifications: TuiNotificationSettings,

    /// Enable ASCII animations and shimmer effects in the TUI.
    pub animations: bool,

    /// Show startup tooltips in the TUI welcome screen.
    pub show_tooltips: bool,

    /// Persisted startup availability NUX state for model tooltips.
    pub model_availability_nux: ModelAvailabilityNuxConfig,

    /// Start the composer in Vim mode (`Normal`) by default.
    pub tui_vim_mode_default: bool,

    /// Start the TUI in raw scrollback mode for copy-friendly transcript output.
    pub tui_raw_output_mode: bool,

    /// Start the TUI in the specified collaboration mode (plan/default).

    /// Controls whether the TUI uses the terminal's alternate screen buffer.
    ///
    /// This is the same `tui.alternate_screen` value from `config.toml`.
    /// - `auto` (defau<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
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
          <h1>URL:https://raw.githubusercontent.com/openai/codex/main/codex-rs/core/src/config/mod.rs</h1>
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