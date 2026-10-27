# Parent integration review

No host file has been edited. `integration/host.patch` is generated against exact
Git object `ea100cee3f69711de12485ba061950c5fc65258f`. Review it separately; do not
apply it to a dirty host blindly. Full host build and live GUI remain unrun.

## Surfaces actually used

All three commands render Form.TextField. Base additionally uses Form.Dropdown,
Dropdown.Item, Separator and Description; status uses Description. Action IDs are
opaque revisioned strings. All content is ordinary React text/input data, including
URLs, entities, angle brackets and NUL. No Detail, Grid, remote image, user Markdown,
arbitrary HTML, List dropdown, or browser surface is used by this product. A
Detail primary callback/MouseEvent patch and List controlled-value patch are
therefore deliberately absent. Actual pinned List parsers still get negative QA.

The Form patch sends every action the complete current draft, plus `_inputRevision`
and `_lastChangedField`. Each field event carries its value and that same metadata.
The adapter accepts increasing field revisions from older renders, rejecting
reordered events; copy action IDs themselves remain one-use and revision-bound.
Thus A/B/A edits are distinct, and a stale action cannot authorize old output.
A copy arriving before its queued latest field update can prepare the latest
whole-form value. If its action revision is already stale, it fails deliberately;
refresh the current view and invoke the newly exposed action. There is no replay.
The Form patch avoids overwriting newer local drafts with delayed server responses.
It adds actual focused-field callbacks and initial focus for source clipboard
behavior. Configuration routing uses `host.settings.open {surface:settings-extensions}`.

## Identity and assets

Source manifest canonical ID is `com.tezbar.native.converter`; exact replacements
are `com.tezbar.converter` and `raycast.converter`. These are evidence-backed by
own package.json and pinned loved_extensions.json, respectively. Source artifacts
are empty; only parent may add real published target pins/catalog metadata.

Pinned registry code only routes legacy Raycast aliases by catalog slug and
resolves icons relative to the canonical package root, although the executable
package lives under its active version. The patch uses only a successfully
verified active package, an exact declared replacement, and a covered compatible
Rust command for alias routing. It keeps canonical preference/uninstall paths
separate from active-version resource paths. It does not grant routing based on a
prefix or a same-slug match. Damaged/uncovered packages do not claim an alias.

The PNG is the byte-exact Raycast asset from upstream
`6734d85992290066fd31a7f5bee3cca3d3545a1a`, SHA-256
`77a16b3723a810618b4781a19c7bee0fba5910cfba58bff89b7a3830bfc02132`.
All meaningful source command assets are this same PNG. The original metadata
screenshots remain in V1 and provenance fixtures, rather than executable resources.

## Protocol and lifecycle

Schema 3 / rust / process / protocol 1; independent SDK 0.1.1 pinned by HTTPS Git
revision `e955911f0bf5b558f9d3eb5183df0adc95c22af2`. Exact grants are clipboard.read,
clipboard.write and settings.open. Clipboard read must return a string. Write
params are `{text}`. ACK must be an object containing only `ok:true` and optional
string `message`; null, booleans, extra keys and contradictions fail.

Domain state uses the SDK staged-state transaction. A bounded separate session
fence consumes revisions/actions before parsing or awaiting and retains exact
uncertain prepared effects. Cancellation/failure cannot roll that fence back.
Open installs an SDK-owned pending cleanup future holding a drop guard. Cancellation
of open, disposal, EOF and shutdown drop it and remove only its own generation's
fence, even if an aborted open never receives dispose. There are no arbitrary
spawned threads or detached tasks; the pinned SDK owns its documented stdio pump
and scoped task lifecycle. The extra future only holds cleanup authority, no work.

No domain persistence exists in the source, so no private store or migration is
introduced. Corrupt-store/symlink-store tests are inapplicable to this converter.
Package/host private paths are not read by domain code.

## Evidence boundaries

`tests/host.rs` compiles exact extracted host validator/registry/resource functions;
only verified-package discovery is stubbed. `scripts/host-qa.cjs` executes actual
pinned Form/List parsers and patched Form/surface callbacks. These narrow tests do
not compile the whole app or prove production installation. `scripts/host-patch.py`
rebuilds the patch; `scripts/host-fixtures.py` hashes/extracts tested functions.
Parent must review/apply/rebase, build the host, publish selected binaries, and
perform clean-profile platform GUI acceptance separately.
