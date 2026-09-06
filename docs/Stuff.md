


# 1
- [x] First Gameplay loop Milestone:

- The Scout Ship can *Secure* planets by gaining *Influence* over the planet by spending a number of turns, determined in the Planet's *Details Page*, within 200u of the planet while no enemey is also within 200u.
  Accumulated influence decays by the amount specified in the planet's details page if no ship is within range.
- A *Secured* planet generates a number of *Resources* determined in it's details page
- Spend resources to build more ships from your home planet and send them out to secure more planets

**What we need to build to get there**

1. Planet Details Page
        - Clicking a planet spawns a popup panel above the planet with the following information:
                - Planet's name
                - Current ownership (Farlight, Starfall, Unclaimed)
                - Security
                        - Number of turns a ship must spend within 200u until the planet is secured. Typically 3-5
                        - Influence decay rate while no ship is attempting to secure. Typically 1-2
                - Resource production information
                        - Resource type(s). Typically 1-2
                        - How often the resource is generated. Typically every 1-3 turns
                        - How many of each resource is generated. Typically 1-3? 
                - If planet is Unclaimed or held by the opponent a *Secure Planet* button appears
                        - Progress bar displaying number of turns accumilated and number of turns remaining.
2. Secure planets mechanic
        - See above
3. Planet generate resources per turn mechanic and planet details page
        - See above
4. Resource tracker in the top *Command Strip*
        - Resource icon and current amount owned with amount being generated in parenthesis
                - Initial resources will consist of: Alloys (common) and Advanced Electronics (uncommon)
5. Home planet ship builder mechanic
        - The *Home Planet* details page has a *Shipyard* tab
        - Shipyard tab will have a *Build Ship* Button with resource cost. While a ship is being built a progress bar is displayed
                Initial Ships available: Scout Ship - 10 Alloys 5 Advanced Electornics 
                
