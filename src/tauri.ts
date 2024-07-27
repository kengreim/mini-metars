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

const update_metar_cmd = (id: string): Promise<FetchMetarResponse> =>
  invoke("fetch_metar", { id: id });

const lookup_station_cmd = (id: string): Promise<Station> => invoke("lookup_station", { id: id });

export { update_metar_cmd, lookup_station_cmd };
export type { CloudLayer, MetarDto };
