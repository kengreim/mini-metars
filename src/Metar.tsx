import { Component, createSignal, onCleanup, onMount, Show } from "solid-js";
import { lookupStationCmd, updateAtisLetterCmd, updateMetarCmd } from "./tauri.ts";
import { logIfDev } from "./logging.ts";

interface MetarProps {
  requestedId: string;
  resizeFn: () => Promise<void>;
}

function getRandomInt(min: number, max: number) {
  const minCeiled = Math.ceil(min);
  const maxFloored = Math.floor(max);
  return Math.floor(Math.random() * (maxFloored - minCeiled) + minCeiled); // The maximum is exclusive and the minimum is inclusive
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
  const [atisLetter, setAtisLetter] = createSignal("-");

  // Update handle
  const [metarTimerHandle, setMetarTimerHandle] = createSignal(-1);
  const [letterTimerHandle, setLetterTimerHandle] = createSignal(-1);

  const fetchAndUpdateStation = async () => {
    try {
      logIfDev("Looking up requested ID", props.requestedId);
      let station = await lookupStationCmd(props.requestedId);
      setIcaoId(station.icaoId);
      setDisplayId(station.faaId);
      setValidId(true);
    } catch (error) {
      setDisplayId(props.requestedId);
      console.log(error);
    }
  };

  const updateMetar = async () => {
    if (!validId()) {
      return;
    }

    try {
      logIfDev("Starting update check for id", icaoId());
      let res = await updateMetarCmd(icaoId());
      logIfDev("Retrieved METAR", icaoId(), res);
      let newTimestamp = new Date(res.metar.obsTime);
      if (currentTimestamp() === undefined || newTimestamp > currentTimestamp()!) {
        logIfDev("New METAR found", icaoId());
        setCurrentTimestamp(newTimestamp);
        setAltimeter(res.altimeter.toFixed(2));
        setWind(res.wind_string);
        setRawMetar(res.metar.rawOb);
      } else {
        logIfDev("Fetched METAR same as displayed", icaoId(), currentTimestamp());
      }
    } catch (error) {
      console.log(error);
    }
  };

  const updateAtisLetter = async () => {
    if (!validId()) {
      return;
    }

    try {
      logIfDev("Starting ATIS letter fetch for id", icaoId());
      let res = await updateAtisLetterCmd(icaoId());
      logIfDev("Retrieved ATIS Letter", res);
      setAtisLetter(res);
    } catch (error) {
      console.log(error);
    }
  };

  onMount(async () => {
    try {
      await fetchAndUpdateStation();
      if (validId()) {
        await updateMetar();
        setMetarTimerHandle(setInterval(updateMetar, 1000 * getRandomInt(120, 150)));

        await updateAtisLetter();
        setLetterTimerHandle(setInterval(updateAtisLetter, 1000 * getRandomInt(60, 90)));
      }
    } catch (error) {
      console.log(error);
    }
  });

  onCleanup(() => {
    if (metarTimerHandle() != -1) {
      clearInterval(metarTimerHandle());
    }

    if (letterTimerHandle() != -1) {
      clearInterval(letterTimerHandle());
    }
  });

  return (
    <div
      class="flex flex-col mx-1 select-none cursor-pointer"
      onClick={async () => {
        setShowFullMetar((prev) => !prev);
        await props.resizeFn();
      }}
    >
      <div class="flex font-mono text-sm">
        <div class="w-12">{displayId()}</div>
        <div class="w-8">{atisLetter()}</div>
        <div class="w-16">{altimeter()}</div>
        <div class="flex-grow">{wind()}</div>
      </div>
      <Show when={showFullMetar() && rawMetar() !== ""}>
        <div class="text-xs mb-1 text-gray-400">{rawMetar()}</div>
      </Show>
    </div>
  );
};
