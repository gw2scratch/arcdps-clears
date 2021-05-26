[![Build and test](https://img.shields.io/github/workflow/status/gw2scratch/arcdps-clears/Build%20and%20test?logo=github)](https://github.com/gw2scratch/arcdps-clears/actions/workflows/build-and-test.yml)
[![Website](https://img.shields.io/website?down_message=gw2scratch.com&up_message=gw2scratch.com&url=https%3A%2F%2Fgw2scratch.com)](https://gw2scratch.com/tools/arcdps-clears)
[![Discord](https://img.shields.io/discord/543804828808249374?label=discord&logo=discord&logoColor=white&)](https://discord.gg/rNXRS6ZkYe)

# arcdps clears
A plugin for arcdps which adds a window for quickly checking your current weekly clears in the game.

[![Ingame screenshot](https://i.imgur.com/8GcJjT4.png)](https://gw2scratch.com/tools/arcdps-clears)

This plugin uses the official Guild Wars 2 API to get the clear data, you will need an [API key](https://wiki.guildwars2.com/wiki/API:API_key) (with access to *account* and *progression*).

The plugin uses no actual arcdps combat data, so it is the same as your typical overlay program – no need to worry about breaking any rules.

## Current State

Please note, this is currently an early version. It's already useful to me, so I am releasing it as it is.
However, there are some extra things that I want to add in the future.

- There is currently no support for clears of friends (the tab does nothing right now)
- There is currently no support for non-raid content (sorry, no dungeons yet!)
- There are some secret plans for big new features

## Translations

You can make a custom translation for the text and the short boss names, should they not be to your liking.

To do so, create an `arcdps_lang_clears.json` file in the `addons/arcdps` directory (next to `arcdps.log`, `settings_clears.json` and others).
You should use the [default translation](translations/arcdps_lang_clears.json) as a base.

If you make a translation to a different language, let us know, we will feature it here.

## Contributing

Please let me know before considering contributing any changes. I have plans for certain features that
are pretty complex and may be completely incompatible with what you want to change.

Feature requests and bug reports are welcome!
