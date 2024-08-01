import { invoke } from "@tauri-apps/api/core";

interface CloudLayer {
  cover: string;
  base?: number;
}

interface MetarDto {
  metarId: number;
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
  mostRecent: number;
  lat: number;
  lon: number;
  elev: number;
  prior: number;
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
  wind_string: string;
  altimeter: number;
}

interface Profile {
  name: string;
  stations: string[];
}

const updateMetarCmd = (id: string): Promise<FetchMetarResponse> =>
  invoke("fetch_metar", { id: id });

const lookupStationCmd = (id: string): Promise<Station> => invoke("lookup_station", { id: id });

const updateAtisLetterCmd = (id: string): Promise<string> =>
  invoke("get_atis_letter", { icaoId: id });

const loadProfile = (): Promise<Profile> => invoke("load_profile", {});

const saveProfile = (profile: Profile): Promise<void> =>
  invoke("save_profile", { profile: profile });

export { updateMetarCmd, lookupStationCmd, updateAtisLetterCmd, loadProfile, saveProfile };
export type { CloudLayer, MetarDto, Profile };
