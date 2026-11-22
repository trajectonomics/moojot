# Play Console — Data safety and photo access

Use this when filling the Google Play Data safety form and the Photo and Video Permissions declaration. It matches the app as shipped.

## Data safety

| Question | Answer |
|---|---|
| Does the app collect or share user data? | **Collects** anonymous usage analytics, only after the user opts in (see below). The ledger is not sent to a MooJot server. PostHog receives the events as our service provider, which Play does not count as sharing. |
| Is data encrypted in transit? | Yes. Usage events go over HTTPS to `moojot.trajectonomics.com`, which forwards them to PostHog EU Cloud. The optional AksonOCR request also uses HTTPS. |
| Is collection optional? | Yes. Analytics is off by default. Users opt in with "Share anonymous usage stats" in the welcome tour or My Moo, and can turn it off there. |
| Can users ask for deletion? | They delete records in the app, or uninstall. The ledger is not held off-device. Turning analytics off deletes the anonymous ID on the phone; deletion of past events can be requested through GitHub issues. |
| Financial info | Entered or imported by the user and stored only on the device. Not collected, not shared. |
| Photos | Read to import Thai bank slips. Stored as private copies inside the app. Not shared. |

Declare **Photos and videos** as collected for app functionality, not shared, processed on device.

Declare these as **collected**, **not shared**, **optional** (off until the user opts in), purpose **Analytics**, processed ephemerally: no; linked to the user's identity: no.

- **App activity → App interactions**: screen views, welcome-tour steps, slip scan started/finished/failed, record saved/deleted (kind only), statement imported (counts only), export format, settings changed.
- **Device or other IDs**: a random ID generated on the phone for analytics (not the advertising ID).
- **App info and performance → Other app performance data** is *not* collected: exception capture and session replay are off.

The exact event list is `src/lib/analyticsEvents.ts`. Re-check this form whenever that file changes.

If the optional AksonOCR key stays in the product, also declare that photos may be sent to the OCR provider **only when the user supplies a key**. The default build path does not.

## Photo and video permissions

- Permission: `READ_MEDIA_IMAGES` (and `READ_MEDIA_VISUAL_USER_SELECTED` on Android 14+).
- Core use: scan the user's own Thai bank-slip albums.
- Alternative: the in-app picker uses `ACTION_GET_CONTENT`, so a user can import one slip or one PDF without granting album access.
- `Camera` and `Screenshots` folders are not scanned. Shared albums are off unless the user opts in, and a slip QR is required before OCR.

## Listing assets

- App icon: `src-tauri/gen/android/app/src/main/res/mipmap-*/`
- Feature graphic source: [play-feature-graphic.svg](play-feature-graphic.svg) (1024×500)
- Screenshots: capture 4–6 phone screens in Pink & White and Black & Orange (Home, empty month, Summary, My Moo) on a device. Do not invent them.

## Not in this pipeline

Biometric app lock and a Jetpack Glance home-screen widget were listed as optional. They are not included here.
