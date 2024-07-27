import { Component, createSignal, onCleanup, onMount, Setter, Show } from "solid-js";
import { lookup_station_cmd, update_metar_cmd } from "./tauri.ts";

interface MetarProps {
  requestedId: string;
  heightTrigger: Setter<number>;
}

export const Metar: Component<MetarProps> = (props) => {
  const [icaoId, setIcaoId] = createSignal("");
  const [currentTimestamp, setCurrentTimestamp] = createSignal<Date>();
  const [validId, setValidId] = createSignal(false);

  // UI Display Signals
  const [displayId, setDisplayId] = createSignal("");
  const [altimeter, setAltimeter] = createSignal("");
  const [wind, setWind] = createSignal("");
  const [rawMetar, setRawMetar] = createSignal("");
  const [showFullMetar, setShowFullMetar] = createSignal(false);

  // Update handle
  const [timer, setTimer] = createSignal(-1);

  const fetchAndUpdateStation = async () => {
    try {
      let station = await lookup_station_cmd(props.requestedId);
      setIcaoId(station.icaoId);
      setDisplayId(station.faaId);
      setValidId(true);
    } catch (error) {
      setDisplayId(props.requestedId);
      console.log(error);
    }
  };

  const update = async () => {
    if (!validId()) {
      return;
    }

    try {
      let res = await update_metar_cmd(icaoId());
      let newTimestamp = new Date(res.metar.obsTime);
      if (currentTimestamp() === undefined || newTimestamp > currentTimestamp()!) {
        setCurrentTimestamp(newTimestamp);
        setAltimeter(res.altimeter.toFixed(2));
        setWind(res.wind_string);
        setRawMetar(res.metar.rawOb);
      }
    } catch (error) {
      console.log(error);
    }
  };

  onMount(async () => {
    try {
      await fetchAndUpdateStation();
      if (validId()) {
        await update();
        setTimer(setInterval(update, 1000 * 120));
      }
    } catch (error) {
      console.log(error);
    }
  });

  onCleanup(() => {
    if (timer() != -1) {
      clearInterval(timer());
    }
  });

  return (
    <div
      class="flex flex-col mx-1 select-none cursor-pointer"
      onClick={() => {
        setShowFullMetar((prev) => !prev);
        props.heightTrigger((prev) => prev + 1);
      }}
    >
      <div class="flex font-mono text-sm">
        <div class="w-12">{displayId()}</div>
        <div class="w-16">{altimeter()}</div>
        <div class="flex-grow">{wind()}</div>
      </div>
      <Show when={showFullMetar() && rawMetar() !== ""}>
        <div class="text-xs mb-1 text-gray-400">{rawMetar()}</div>
      </Show>
    </div>
  );
};
