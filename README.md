[![Build and test](https://img.shields.io/github/actions/workflow/status/gw2scratch/arcdps-clears/build-and-test.yml?logo=github)](https://github.com/gw2scratch/arcdps-clears/actions/workflows/build-and-test.yml)
[![Website](https://img.shields.io/website?down_message=gw2scratch.com&up_message=gw2scratch.com&url=https%3A%2F%2Fgw2scratch.com)](https://gw2scratch.com/tools/arcdps-clears)
[![Discord](https://img.shields.io/discord/543804828808249374?label=discord&logo=discord&logoColor=white&)](https://discord.gg/rNXRS6ZkYe)

# arcdps clears
A plugin for arcdps which adds a window for quickly checking your current weekly clears in the game.

This plugin uses the official Guild Wars 2 API to get the clear data, you will need an [API key](https://wiki.guildwars2.com/wiki/API:API_key) (with access to *account* and *progression*).

The plugin uses no actual arcdps combat data, so it is the same as your typical overlay program – no need to worry about breaking any rules.

## Screenshots
[![Ingame screenshot](https://i.imgur.com/mLdc74W.png)](https://gw2scratch.com/tools/arcdps-clears)
[![Friends](https://i.imgur.com/2X3XWZs.png)](https://gw2scratch.com/tools/arcdps-clears)


## Features

- Shows raid clears for your account(s) within the game.
- Shows raid clears of friends if they also use the addon.
- Offers multiple table styles and many other configuration options

### Currently not supported

- non-raid content (sorry, no dungeons or world bosses yet!)
- weekly CM achievement – currently impossible due to [API limitations](https://github.com/gw2-api/issues/issues/2)

## Usage

An [user guide](https://guides.gw2scratch.com/clears/) is available,
with installation steps and descriptions of how to use the plugin.

## Translations

You can make a custom translation for the text and the short boss names, should they not be to your liking.

To do so, create an `arcdps_lang_clears.json` file in the `addons/arcdps` directory (next to `arcdps.log`, `settings_clears.json` and others).
You should use the [default translation](translations/arcdps_lang_clears.json) as a base.

If you make a translation to a different language, let us know, we will feature it here.

## Contributing

Please let me know before considering contributing any changes. I have plans for certain features that
are pretty complex and may be completely incompatible with what you want to change.

Feature requests and bug reports are welcome!
