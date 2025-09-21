import { invoke } from "@tauri-apps/api/core";

interface CloudLayer {
  cover: string;
  base?: number;
}

interface MetarDto {
  icaoId: string;
  receiptTime: string;
  obsTime: string;
  reportTime: string;
  temp?: number;
  dewp?: number;
  wdir?: number | string;
  wspd?: number;
  wgst?: number;
  visib: string | number;
  altim: number;
  slp?: number;
  qcField: number;
  wxString?: string;
  presTend?: number;
  maxT?: number;
  minT?: number;
  maxT24?: number;
  minT24?: number;
  precip?: number;
  pcp3hr?: number;
  pcp6hr?: number;
  pcp24hr?: number;
  snow?: number;
  vertVis?: number;
  metarType: string;
  rawOb: string;
  mostRecent?: number;
  lat: number;
  lon: number;
  elev: number;
  prior?: number;
  name: string;
  clouds: CloudLayer[];
}

interface Station {
  icaoId: string;
  iataId: string;
  faaId: string;
  wmoId: string;
  lat: number;
  lon: number;
  elev: number;
  site: string;
  state: string;
  country: string;
  priority: number;
}

interface FetchMetarResponse {
  metar: MetarDto;
  windString: string;
  altimeter: { inHg: number; hPa: number };
}

interface FetchAtisResponse {
  letter: string;
  texts: string[];
}

interface Profile {
  name: string;
  stations: string[];
  showInput: boolean;
  showTitlebar: boolean;
  window?: Window;
  units: "inHg" | "hPa";
  hideAirportIfMissingAtis: boolean;
}

interface Window {
  state: "Normal" | "Maximized" | "FullScreen";
  position: { x: number; y: number };
  size: { width: number; height: number };
  scaleFactor: number;
}

interface Settings {
  loadMostRecentProfileOnOpen: boolean;
  mostRecentProfile?: string;
  alwaysOnTop: boolean;
  autoResize: boolean;
}

interface InitialSettingsLoad {
  settings: Settings;
  profile?: Profile;
}

const updateMetarCmd = (id: string): Promise<FetchMetarResponse> =>
  invoke("fetch_metar", { id: id });

const lookupStationCmd = (id: string): Promise<Station> => invoke("lookup_station", { id: id });

const updateAtisCmd = (id: string): Promise<FetchAtisResponse> =>
  invoke("get_atis", { icaoId: id });

const loadProfileCmd = (): Promise<Profile> => invoke("load_profile", {});

const saveProfileCmd = (profile: Profile): Promise<void> =>
  invoke("save_current_profile", { profile: profile });

const saveProfileAsCmd = (profile: Profile): Promise<void> =>
  invoke("save_profile_as", { profile: profile });

const loadSettingsCmd = (): Promise<Settings> => invoke("load_settings", {});

const loadSettingsInitialCmd = (): Promise<InitialSettingsLoad> =>
  invoke("load_settings_initial", {});

const saveSettingsCmd = (settings?: Settings): Promise<void> =>
  invoke("save_settings", { settings: settings });

const initializeDatafeedCmd = (): Promise<void> => invoke("initialize_datafeed", {});

export {
  updateMetarCmd,
  lookupStationCmd,
  updateAtisCmd,
  loadProfileCmd,
  saveProfileCmd,
  saveProfileAsCmd,
  loadSettingsCmd,
  loadSettingsInitialCmd,
  saveSettingsCmd,
  initializeDatafeedCmd,
};
export type { CloudLayer, MetarDto, Profile, Settings, InitialSettingsLoad };
