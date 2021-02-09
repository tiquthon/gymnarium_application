# Availables

This file lists the availability of combined components.
It should help keeping the correct structure in code `./src/availables.rs`.

## Environment x Agent

|     | GymMountainCar | CodeBulletAiLearnsToDrive |
| --- | --- | --- |
| Random | yes | yes |
| Input | yes | yes |

## Environment x Visualiser

|     | GymMountainCar | CodeBulletAiLearnsToDrive |
| --- | --- | --- |
| None | yes | yes |
| PistonIn2d | yes | yes |

## Environment x Exit Condition

|     | GymMountainCar | CodeBulletAiLearnsToDrive |
| --- | --- | --- |
| EpisodesSimulated | yes | yes |
| VisualiserClosed | yes | yes |

## Agent x Visualiser

|     | Random | Input |
| --- | --- | --- | --- |
| None | yes | **no** |
| PistonIn2d | yes | yes |

## Agent x Exit Condition

|     | Random | Input |
| --- | --- | --- | --- |
| EpisodesSimulated | yes | yes |
| VisualiserClosed | yes | yes |

## Visualiser x Exit Condition

|     | None | PistonIn2d |
| --- | --- | --- |
| EpisodesSimulated | yes | yes |
| VisualiserClosed | **no** | yes |
