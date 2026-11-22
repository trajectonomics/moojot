# Android release

MooJot 1.0.0 is versioned in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. The Android `versionName` comes from the Tauri build. `versionCode` is `major * 10000 + minor * 100 + patch` unless `tauri.android.versionCode` is set (1.0.0 → 10000).

## Signing key

Create the keystore once, on a machine you control. Do not commit it.

```bash
keytool -genkeypair -v \
  -keystore moojot-release.jks \
  -alias moojot \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000
```

`src-tauri/gen/android/app/build.gradle.kts` signs release builds when these are set. Environment variables win over `keystore.properties`.

| Env | `keystore.properties` |
|---|---|
| `ANDROID_KEYSTORE_PATH` | `storeFile` |
| `ANDROID_KEYSTORE_PASSWORD` | `storePassword` |
| `ANDROID_KEY_ALIAS` | `keyAlias` |
| `ANDROID_KEY_PASSWORD` | `keyPassword` |

`keystore.properties` may live in `src-tauri/gen/android/` or `src-tauri/`. `*.jks`, `*.keystore`, and `keystore.properties` are gitignored.

GitHub Actions secrets for tag builds:

- `ANDROID_KEYSTORE_BASE64` — `base64 -w0 moojot-release.jks`
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS` (for example `moojot`)
- `ANDROID_KEY_PASSWORD`

## Build

Use Android NDK r28 or newer so native libraries are 16 KB aligned.

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
pnpm tauri android build --apk --target aarch64
pnpm tauri android build --aab --target aarch64
```

Check alignment after the build (every `LOAD` segment should be 0x4000 or larger):

```bash
READELF="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-readelf"
unzip -q -d /tmp/moojot-apk path/to/app.apk
find /tmp/moojot-apk/lib -name '*.so' -print0 |
  xargs -0 -n1 "$READELF" -l |
  awk '/LOAD/ { if (strtonum($NF) < 16384) bad = 1 } END { exit bad }'
```

## CI

`.github/workflows/ci.yml` runs on every pull request and every push to `main`. It runs `pnpm check`, `pnpm test`, `pnpm build`, `pnpm test:e2e` (headless Chrome) and `cargo test --lib`.

## Releasing

`.github/workflows/release-android.yml` builds the arm64 APK and AAB. Before it does, it checks that the versions agree, runs the typecheck and the unit and Rust tests, and checks 16 KB alignment.

**Dry run** (publishes nothing): Actions → `release-android` → Run workflow, or

```bash
gh workflow run release-android.yml --ref main
```

The APK, AAB and `SHA256SUMS` are attached to the run as an artifact for 7 days. Without the keystore secrets the dry-run APK is unsigned, so it won't install until you sign it.

**Publish a release:**

1. Set the same version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`, and merge that change.
2. Tag `main` and push the tag:

   ```bash
   git tag v1.0.0 origin/main
   git push origin v1.0.0
   ```

3. The workflow signs the build with the release keystore (it fails if the secrets are missing), checks the signature, and publishes a GitHub Release with:

| Asset | What it is |
|---|---|
| `moojot-android-arm64.apk` | The app, for direct install |
| `moojot-android-arm64.aab` | The Google Play upload |
| `SHA256SUMS` | Checksums of both |

### Stable download links

The asset names don't change between releases, so these links always point at the newest release. Use them on the landing page:

- APK: `https://github.com/trajectonomics/moojot/releases/latest/download/moojot-android-arm64.apk`
- Checksums: `https://github.com/trajectonomics/moojot/releases/latest/download/SHA256SUMS`
- Release page: `https://github.com/trajectonomics/moojot/releases/latest`

The build is arm64 only (`aarch64`), which covers current Android phones.

## Play Store

- Privacy policy: [../privacy-policy.md](../privacy-policy.md)
- Data safety and photo declaration: [play-data-safety.md](play-data-safety.md)
- Upload the `.aab` from the release build. The GitHub Release is the APK channel.
