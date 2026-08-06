#!/usr/bin/env python3
"""Reconstruct chunked files. Run from repo root after git clone."""
import os, base64, json

def main():
    # Read chunk_info (may be split into parts)
    parts = []
    for f in sorted(os.listdir("agents")):
        if f == "chunk_info.json":
            with open(os.path.join("agents", f), "r", encoding="utf-8") as fh:
                chunk_info = json.load(fh)
            break
        if f.startswith("chunk_info.__part") and f.endswith("__.json"):
            with open(os.path.join("agents", f), "r", encoding="utf-8") as fh:
                parts.append(json.load(fh))
    else:
        if parts:
            chunk_info = {}
            for p in parts:
                chunk_info.update(p)
        else:
            print("ERROR: No chunk_info found")
            return

    reconstructed = 0
    failed = 0

    for original_path, info in chunk_info.items():
        chunks = info["chunks"]
        encoding = info["encoding"]
        os.makedirs(os.path.dirname(original_path), exist_ok=True)

        if os.path.exists(original_path) and "__part" not in original_path:
            continue

        try:
            if encoding == "utf-8":
                content = ""
                for cp in chunks:
                    with open(cp, "r", encoding="utf-8") as f:
                        content += f.read()
                with open(original_path, "w", encoding="utf-8") as f:
                    f.write(content)
            else:
                b64 = ""
                for cp in chunks:
                    with open(cp, "r", encoding="utf-8") as f:
                        b64 += f.read()
                with open(original_path, "wb") as f:
                    f.write(base64.b64decode(b64))

            for cp in chunks:
                if cp != original_path and os.path.exists(cp):
                    os.unlink(cp)
            reconstructed += 1
            print(f"OK: {original_path}")
        except Exception as e:
            failed += 1
            print(f"FAIL: {original_path} - {e}")

    # Clean empty dirs
    for root, dirs, files in os.walk("agents", topdown=False):
        for d in dirs:
            dp = os.path.join(root, d)
            try:
                if not os.listdir(dp):
                    os.rmdir(dp)
            except:
                pass

    # Clean up chunk_info parts
    for f in os.listdir("agents"):
        if f.startswith("chunk_info.__part") and f.endswith("__.json"):
            try:
                os.unlink(os.path.join("agents", f))
            except:
                pass

    print(f"\nReconstructed: {reconstructed}, Failed: {failed}")

if __name__ == "__main__":
    main()
