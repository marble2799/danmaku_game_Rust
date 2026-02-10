# Simple Danmaku Shooting Game

## develop memo
### 2026/02/06
Verify that it works corrently with `cargo run`. However, `trunk serve` does not work well(only black window is shown).
-> It works with `FireFox` !! Maybe Chrome with Ubuntu denies GPU because a lot of crash reports are reported.

### 2026/02/07
- Impliment moving function
- Impliment shooting function. Push `space` key !!
- Impliment enemy spawn and moving with Acceleration.
- add Collider in player, bullet and enemy.(ONLY ADD!!)

### 2026/02/09
- Impliment Some struct
  - TeamSide
  - Status
  - Score(need showing function!!)
- Impliment all collision
  - bullet and {Player, Enemy}
  - Player and Enemy
- refact some struct(add impl)
- make Simple GameOver

### 2026/02/10
- Impliment Enemy shooting
- Impliment GameStart and GameOver State(Text Window)
    - can restart
- Impliment ScoreBoard
## Fist version is completed !!