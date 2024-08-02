# Mini METARs
Mini METARs is a micro-utility to display up-to-date METAR information (primarily altimeter and wind direction + speed, with full METAR toggle-able) and VATSIM ATIS code for a number of user-inputted airports/stations in a minimal on-top window.

Built with Tauri, with a Rust backend for METAR fetching and a SolidJS frontend for UI actions.

![image](https://github.com/user-attachments/assets/989b103b-64f5-4d43-89ef-c9c60962ddd0)

## FAQ

**How often do METARs update**?

* Each airport/station checks for a METAR update every 2 to 2.5 minutes, with the value slightly randomized to prevent "clumping" of requests.

**How often do VATSIM ATIS codes update?**

* Each airport/station checks for a VATSIM ATIS code update ever 1 to 1.5 minutes.

**What if an airport has separate arrival and departure ATIS?**

* Both codes will be displayed in the format "`ARRIVAL_CODE`/`DEPARTURE_CODE`"

**Does it support "profiles" or lists of airports so that I don't need to manually enter several each time I start the program?**

* Yes, you can save an existing set of stations/airports using `Ctrl+S`, which will open the native system file dialog, and open a profile using `Ctrl+O`
