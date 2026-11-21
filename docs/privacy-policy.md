# MooJot Privacy Policy

MooJot is a personal income and expense tracker. The ledger lives on your device.

## What stays on the device

- Accounts, categories, records, budgets, and settings are stored in a local SQLite database.
- Bank-slip photos you import are copied into the app's private files.
- Credit-card statement PDFs are decrypted and read on the device. The password is used only for that import and is not saved.
- MooJot has no account server and does not send analytics.

## Photos

Scanning bank albums uses Android photo access (`READ_MEDIA_IMAGES`) so MooJot can read the albums you enable. You can instead pick a single slip with the system file picker. Shared albums stay off until you turn them on, and even then MooJot only keeps images that pass the local slip check.

## Optional cloud OCR

Thai text is recognized on the device. If you type an AksonOCR API key into settings, slip images are sent to that service for recognition. Leave the key empty and nothing is uploaded.

## Sharing

Export and the Android share sheet send a file only when you ask. MooJot does not upload that file itself.

## Contact

Questions about this policy can be opened as an issue on the MooJot repository.
