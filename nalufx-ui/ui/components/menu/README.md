<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg"
alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Menu Component

The `Menu` component is a flexible and customizable UI element that provides menu functionality. It includes a menu button and supports various properties and behaviors to control its appearance and actions.

## Features

- **Visibility Control**: The `Menu` component can be shown or hidden programmatically.
- **State Management**: Supports states for open and closed menus with animations.
- **Customizable Appearance**:
  - Width and Height: Adjust the dimensions of the menu.
  - Background: Set the background color for the menu.
- **Callbacks**: Provides callbacks for opened and closed states.
- **Mobile Support**: Includes a `MobileMenu` component for responsive design.

## Usage

To use the `Menu` component, import it and configure it as needed:

```slint
import { Menu } from "menu.slint";

// Example usage
export component Example inherits Rectangle {
    Menu {
        start-y: 50px
        end-y: 200px
        menu-width: 300px
        menu-height: 400px
        stays-open: true
        menu-button-visible: true
        opened => {
            console.log("Menu opened");
        }
        closed => {
            console.log("Menu closed");
        }
    }
}
```

### Properties

- **menu-button-visible**: Controls the visibility of the menu button.
- **start-y**: The starting Y position of the menu.
- **end-y**: The ending Y position of the menu.
- **stays-open**: Determines if the menu stays open.
- **menu-width**: The width of the menu.
- **menu-height**: The height of the menu.
- **open**: Indicates whether the menu is open.

### Callbacks

- **opened**: Called when the menu is opened.
- **closed**: Called when the menu is closed.

### Functions

- **hide-button()**: Hides the menu button.
- **open-menu()**: Opens the menu.
- **hide()**: Hides the menu.

## MobileMenu Component

The `MobileMenu` component provides a responsive menu for mobile devices.

### Properties

- **open**: Indicates whether the mobile menu is open.
- **end-y**: The ending Y position of the mobile menu.
- **menu-x**: The X position of the mobile menu.
- **menu-width**: The width of the mobile menu (default: 200px).

### Functions

- **open-menu()**: Opens the mobile menu.
- **hide()**: Hides the mobile menu.

### Usage

To use the `MobileMenu` component, import it and configure it as needed:

```slint
import { MobileMenu } from "menu.slint";

// Example usage
export component MobileExample inherits Rectangle {
    MobileMenu {
        end-y: 300px
        menu-x: 50px
        menu-width: 250px
        open: true
    }
}
```

## Installation

To install and use the `Menu` component, make sure to include it in your project dependencies and import it as shown in the usage examples.
