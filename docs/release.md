# Releasing Kontroll

Pushing a `v*` tag runs `.github/workflows/release.yml`. It builds and signs the
binaries, creates the GitHub release, publishes the same archives to
CrabNebula Cloud, where `kontroll upgrade` looks for updates, and copies them to
https://downloads.zsa.io.

Kontroll builds against zapp's crates by path (`../zapp/...`). The workflow
checks this repo out into `kontroll/` and `zsa/zapp` into `zapp/`, at the commit
in `ZAPP_REF` at the top of the workflow. When `Cargo.lock` is updated against a
newer zapp, update `ZAPP_REF` to that commit too: the build runs with
`--locked`.

Tags before 2.0.0 have no `v` (`1.0.3`). They do not trigger the workflow.

## Before tagging

- Set `version` in `Cargo.toml` to the tag without its `v`. The `preflight` job
  fails otherwise: `kontroll upgrade` compares the server's version with the
  version built into the binary.
- Add `release-notes/<tag>.md` (for example `release-notes/v2.0.0.md`). The
  `preflight` job fails if it is missing or empty. The file is the body of the
  GitHub release and the notes `kontroll upgrade` shows.

## Secrets

All secrets live in the `release` environment (Settings → Environments →
`release`), none at repo level. Every job that reads one runs in that
environment.

| Secret | What it is |
| --- | --- |
| `AZURE_CLIENT_ID` | Client ID of the Azure app registration that signs Windows binaries (the same one zapp uses) |
| `AZURE_TENANT_ID` | ZSA's Azure tenant ID |
| `APPLE_CERTIFICATE` | Developer ID Application certificate, `.p12`, base64 |
| `APPLE_CERTIFICATE_PASSWORD` | Password of that `.p12` |
| `APPLE_ID` | Apple ID used for notarization |
| `APPLE_ID_PASSWORD` | App-specific password of that Apple ID |
| `APPLE_TEAM_ID` | ZSA's Apple team ID |
| `CN_API_KEY` | CrabNebula Cloud API key with read/write, from the org's API keys tab |
| `KONTROLL_UPDATER_PRIVATE_KEY` | Contents of `~/.tauri/kontroll.key`, the updater signing key |
| `DOWNLOADS_CLOUDFLARE_API_TOKEN` | Cloudflare API token with R2 write access to the `zsa-downloads` bucket (Object Read & Write, scoped to that bucket) |
| `CLOUDFLARE_ACCOUNT_ID` | The Cloudflare account ID that owns `zsa-downloads` |
| `DOWNLOADS_DISPATCH_TOKEN` | Fine-grained GitHub token limited to `zsa/downloads`, with Contents read and write |

```sh
gh secret set KONTROLL_UPDATER_PRIVATE_KEY --env release --repo zsa/kontroll < ~/.tauri/kontroll.key
```

The key has no password. Its public half (`~/.tauri/kontroll.key.pub`) is
`PUBLIC_KEY` in `src/upgrade.rs`. Back the private key up: installed copies
only accept archives signed with it.

## CrabNebula Cloud

Create the app `io-zsa-kontroll` in the `zsa-technology-labs-inc` org before
the first release. `CN_APPLICATION` in the workflow is the org/app slug:
`zsa-technology-labs-inc/io-zsa-kontroll`.

The release job signs each archive with the Tauri signer (`<archive>.sig`),
drafts a Cloud release with the notes file, uploads the archives, creates the
GitHub release, then publishes the Cloud release.

| Archive | Update platform | Public platform |
| --- | --- | --- |
| `kontroll-linux-x86_64.tar.gz` | `linux-x86_64` | `cli-linux-x86_64` |
| `kontroll-linux-aarch64.tar.gz` | `linux-aarch64` | `cli-linux-aarch64` |
| `kontroll-windows-x86_64.zip` | `windows-x86_64` | `cli-windows-x86_64` |
| `kontroll-windows-aarch64.zip` | `windows-aarch64` | `cli-windows-aarch64` |
| `kontroll-macos-universal.tar.gz` | `darwin-x86_64` | `cli-darwin-x86_64` |
| `kontroll-macos-universal.tar.gz` | `darwin-aarch64` | `cli-darwin-aarch64` |

The per-arch macOS archives stay on GitHub only.

The update endpoint is
`https://cdn.crabnebula.app/update/zsa-technology-labs-inc/io-zsa-kontroll/<platform>/<version>`.
It answers 204 when there is nothing to offer and caches replies for up to an
hour. It also answers 200 when the installed version is newer than the
published one, so `kontroll upgrade` only installs a strictly newer version.

## downloads.zsa.io

After the Cloud release is published, the `downloads` job copies the public
files to https://downloads.zsa.io. It:

1. takes the universal macOS archive, both Windows zips and both Linux archives
   (not the per-arch macOS archives or the `.sig` files),
2. uploads each to the `zsa-downloads` R2 bucket as
   `files/kontroll/<version>/<file name>`,
3. tells the `zsa/downloads` repo about the release (`product: kontroll`,
   version, date, the notes file, and each file's size and SHA-256), which
   updates the site.

The `zsa/downloads` site must know the `kontroll` product to list it.

It is a separate job. If it fails, the run shows red but the Cloud release, the
GitHub release and the updater are already live and unaffected. Fix the cause
(usually an expired or missing secret), then open the run and choose **Re-run
failed jobs**: only `downloads` runs again. Uploads overwrite, so a re-run is
safe.

## How `kontroll upgrade` installs

It checks the signature against the archive before unpacking anything, then
replaces its own executable. It refuses before downloading when it is installed
by Nix, Homebrew, mise or a Linux package (under `/usr`, except `/usr/local`),
or when its folder is not writable.

`kontroll upgrade` is behind the default `upgrade` cargo feature. Library users
who do not want its dependencies (reqwest, rustls, zip, ...) can depend on
`kontroll = { version = "2", default-features = false, features = ["ble"] }`.

## Testing an upgrade end to end

1. Publish a release, say `v2.0.1`.
2. Build a Kontroll that reports an older version: set `version` in
   `Cargo.toml` below the published one (do not commit it), run
   `cargo build --release`, and copy `target/release/kontroll` to a writable
   folder outside the repo. Releases before `v2.0.0` have no
   `kontroll upgrade`.
3. Run `./kontroll upgrade --check` from that copy. It should list the new
   version and its notes.
4. Run `./kontroll upgrade`, answer `y`, then `./kontroll --version`.
5. Optionally check the endpoint by hand:
   `curl -i https://cdn.crabnebula.app/update/zsa-technology-labs-inc/io-zsa-kontroll/linux-x86_64/2.0.0`.

## Lock down the release environment

This repo is public. The workflow only runs on `v*` tags, and the secrets are
only handed to jobs in the `release` environment. Limit that environment to
`v*` tags so no other ref can reach them.

1. Create the environment, if it does not exist, and switch it to custom
   deployment rules:

   ```sh
   gh api --method PUT repos/zsa/kontroll/environments/release \
     -F 'deployment_branch_policy[protected_branches]=false' \
     -F 'deployment_branch_policy[custom_branch_policies]=true'
   ```

2. Allow only `v*` tags:

   ```sh
   gh api --method POST repos/zsa/kontroll/environments/release/deployment-branch-policies \
     -f name='v*' -f type=tag
   ```

   Check it: `gh api repos/zsa/kontroll/environments/release/deployment-branch-policies`
   should list one policy, `v*` of type `tag`, and no branch policies.

3. Optional: require a reviewer before any job in the environment runs. Every
   release then waits for an approval in the Actions tab (the `build`,
   `release` and `downloads` jobs each ask). The PUT replaces the environment's
   settings, so it repeats the deployment rule:

   ```sh
   id=$(gh api users/<github-login> --jq .id)
   gh api --method PUT repos/zsa/kontroll/environments/release --input - <<JSON
   {
     "reviewers": [{ "type": "User", "id": $id }],
     "deployment_branch_policy": { "protected_branches": false, "custom_branch_policies": true }
   }
   JSON
   ```

   The same is under Settings → Environments → `release` → Required reviewers.

4. Optional: restrict who can push `v*` tags with a tag ruleset (Settings →
   Rules → Rulesets → New tag ruleset, target `v*`, restrict creations).

5. Azure: on the app registration behind `AZURE_CLIENT_ID`, add a federated
   credential that trusts this repo's `release` environment. This repo uses the
   default OIDC subject (`gh api repos/zsa/kontroll/actions/oidc/customization/sub`
   shows `"use_default": true`), so the subject is
   `repo:zsa/kontroll:environment:release`:

   ```sh
   az ad app federated-credential create --id <AZURE_CLIENT_ID> --parameters '{
     "name": "kontroll-release",
     "issuer": "https://token.actions.githubusercontent.com",
     "subject": "repo:zsa/kontroll:environment:release",
     "audiences": ["api://AzureADTokenExchange"]
   }'
   ```

   The app also needs the `Artifact Signing Certificate Profile Signer` role on
   the `ZSASigningProfile` certificate profile (account
   `zsa-code-signing-account`). The app registration zapp signs with has it, so
   reusing that one only needs the federated credential.
