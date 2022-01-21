# arcdps clears changelog
This is the full changelog of **arcdps clears**.

## arcdps clears 1.0
The **Friends** update is finally here.

#### New features
- Added support for seeing clears of your friends:
    - requires friends to also use the addon,
    - clears may be set as public or friends only (for a configured list of friends) for each API key in the API key management window,
    - friend clears have a separate style from the *My clears* tab,
    - see the *Friends* tab or the [guide](https://guides.gw2scratch.com/clears/friends/how-to.html) for more details.
- Added a [user guide](https://guides.gw2scratch.com/clears/).
- Added a "show title bar" setting for the Clears window.
- Added a "show background" setting for the Clears window.
- Added a setting for changing the clear check interval (defaults to every 3 minutes).
- Shown accounts may be toggled in the right-click menu.
- Added settings to the Extensions tab in arcdps settings (in addition to the current location).
- Added an *About* window.

#### Changes
- Settings not applicable to the current clear layout are hidden.

#### Fixes
- Raid reset is now handled correctly.
- Opening browser windows no longer causes freezes when running Guild Wars 2 in wine.

## arcdps clears 0.3.1
#### Fixes
- We kinda forgot to update the version number to 0.3.0, so here we go with another release that just bumps the number, and the addon should no longer keep telling you it's out of date every time it's launched!

## arcdps clears 0.3.0
This is a small release adding a couple of style options and simplifying color settings.

#### New features
- Added a new account list style that allows you to collapse clears of individual accounts (useful if you have multiple keys added)
- Added an option to hide column names
- Added an option to hide row names
- Added a button to reset style settings

#### Changes
- Color settings now show the alpha (transparency) slider by default to make it easier to change
- Changed the default unfinished color to a semi-transparent red. Only affects new users, you can use the reset style button to check this out.

#### Preview
![image](https://user-images.githubusercontent.com/998408/121264224-22011a00-c8b7-11eb-8738-5b4da11942d9.png)

## arcdps clears 0.2.0
#### New features
- Added support for showing more than one account (support for alts!)
- Added color settings
- Added a new layout style: wings in columns
- Added a new layout style: one row per account
- Added a keybind to open the clears window (defaults to arcdps modifiers + C)
- Update notifications, letting you know there is a new version available (can be disabled)
- There is now a tutorial good enough for someone who has never used an API key before

#### Changes
- Changed the default colors to something that does not burn your eyes
- Moved the checkbox next to the other ones in the arcdps menu
- Windows can now be closed by pressing escape (can be disabled)
- Windows can now be shown in loading screens and character selection (can be disabled)
- Description markers in settings are a bit more visible (hover over them for details about an option)

## arcdps clears 0.1.0
Please note, this is currently a very early version. It's already useful to me, so I am releasing it as it is. However, there are some extra things that I want to add in the future.

- There is currently no support for clears of alt accounts
- There is currently no support for clears of friends (the tab does nothing right now)