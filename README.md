# Luna

## Project Luna
This project stems from some of my prior project ideas. The origins is the want for a Rainmeter like program, but with a bit easier skin creation system along with my ideas and changes. A side desire with this project is possible future cross platform support.

This project started ast he virt crate which was going to use Vulkan as its back end rendering but the nature of Vulkan was found to be a bit over kill and not well fit for a desktop organizer / visualizer style program. After that there was look in to the rust gfx crate but was found that transparent windows were not natively supported for both linux/unix and macos which made using a complicated cross platform api like gfx overkill. In the end it was desired on using pure winapis for drawing and user input stuff. Below will be a list of TODOs and other wants for this project up until its 0.1.0 release.

## Current In Progress
- Core structures
- Multi Window
- Event Handling

## TODOs
- Toml skin loading
- Image loading
- Basic geometry
- User given commands

## Stretch
- Skin builder
- Default provided comment skins
- Skin library for people to share skins