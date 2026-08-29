interface Preferences {
  autoPaste: boolean;
  asciiCommaSeparated: boolean;
  advanceView: boolean;
  advanceViewLocation: string;
}
function safeGetPrefs(): Preferences {
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const api = require("@raycast/api") as { getPreferenceValues: <T>() => T };
    return api.getPreferenceValues<Preferences>();
  } catch {
    return { autoPaste: true, asciiCommaSeparated: true, advanceView: true, advanceViewLocation: "Bottom" };
  }
}
const prefs = safeGetPrefs();
export const autoPaste = prefs.autoPaste;
export const asciiCommaSeparated = prefs.asciiCommaSeparated;
export const advanceView = prefs.advanceView;
export const advanceViewLocation = prefs.advanceViewLocation;
