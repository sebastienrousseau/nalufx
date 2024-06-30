# Button Component

The `Button` component is a versatile UI element designed to handle user interactions in a clean and consistent manner. This document provides a detailed overview of the properties, events, and usage of the `Button` component.

## Table of Contents

1. [Overview](#overview)
2. [Properties](#properties)
3. [States](#states)
4. [Events](#events)
5. [Usage](#usage)
6. [Customization](#customization)
7. [Examples](#examples)
8. [Related Components](#related-components)

## Overview

The `Button` component inherits from `ButtonCore` and extends its functionality with theme-specific styling and additional properties. It is designed to be highly customizable and accessible, making it suitable for a variety of applications.

## Properties

### Common Properties

- **`buttonText`**: The text displayed on the button.
  - Type: `string`
  - Default: `"Button"`
  - Example: `buttonText: "Submit"`

- **`isEnabled`**: Determines if the button is enabled.
  - Type: `bool`
  - Default: `true`
  - Example: `isEnabled: false`

- **`isPlain`**: Determines if the button has a plain background.
  - Type: `bool`
  - Default: `false`
  - Example: `isPlain: true`

- **`isLoading`**: Determines if the button is in loading state.
  - Type: `bool`
  - Default: `false`
  - Example: `isLoading: true`

- **`textColor`**: The color of the button text.
  - Type: `brush`
  - Default: `black`
  - Example: `textColor: Theme.palette.primary`

- **`hoverTextColor`**: The color of the button text when hovered.
  - Type: `brush`
  - Default: `textColor`
  - Example: `hoverTextColor: Theme.palette.secondary`

- **`loadingTextColor`**: The color of the button text when in loading state.
  - Type: `brush`
  - Default: `gray`
  - Example: `loadingTextColor: Theme.palette.warning`

- **`fontSize`**: The font size of the button text.
  - Type: `length`
  - Default: `14px`
  - Example: `fontSize: 16px`

- **`fontWeight`**: The font weight of the button text.
  - Type: `int`
  - Default: `400`
  - Example: `fontWeight: 700`

- **`borderWidth`**: The width of the button border.
  - Type: `length`
  - Default: `1px`
  - Example: `borderWidth: 2px`

- **`borderRadius`**: The radius of the button border corners.
  - Type: `length`
  - Default: `5px`
  - Example: `borderRadius: 10px`

- **`cursor`**: The mouse cursor when hovering over the button.
  - Type: `MouseCursor`
  - Default: `pointer`
  - Example: `cursor: MouseCursor.default`

## States

- **`isLoading`**: When `isLoading` is `true`, the button displays the `loadingText` and changes the text color to `loadingTextColor`.
- **`isDisabled`**: When `isEnabled` is `false`, the button's opacity is reduced, and the cursor changes to indicate it is not clickable.

## Events

- **`clicked`**: This event is triggered when the button is clicked.
  - Example:

    ```slint
    Button {
        buttonText: "Click Me";
        clicked => {
            // Handle click event
        }
    }
    ```

## Usage

To use the `Button` component, import it and include it in your layout. Customize its properties to fit your design requirements.

### Basic Usage

```slint
import { Button } from "button/button.slint";

Button {
    buttonText: "Submit";
    clicked => {
        // Handle submit action
    }
}
```

### Disabled Button

```slint
Button {
    buttonText: "Disabled";
    isEnabled: false;
}
```

### Loading Button

```slint
Button {
    buttonText: "Load Data";
    isLoading: true;
    loadingText: "Loading...";
}
```

## Customization

The `Button` component can be customized to match your application's theme and design language.

### Custom Text Color

```slint
Button {
    buttonText: "Custom Color";
    textColor: Theme.palette.primary;
}
```

### Custom Font Size and Weight

```slint
Button {
    buttonText: "Custom Font";
    fontSize: 18px;
    fontWeight: 700;
}
```

### Custom Border

```slint
Button {
    buttonText: "Custom Border";
    borderWidth: 2px;
    borderRadius: 10px;
}
```

## Examples

Here are some examples demonstrating different configurations of the `Button` component.

### Example 1: Primary Button

```slint
Button {
    buttonText: "Primary";
    textColor: Theme.palette.primary;
    fontSize: 16px;
    fontWeight: 600;
}
```

### Example 2: Secondary Button

```slint
Button {
    buttonText: "Secondary";
    textColor: Theme.palette.secondary;
    fontSize: 14px;
    fontWeight: 400;
}
```

## Related Components

- **`ButtonCore`**: The base component for buttons, providing common properties and functionality.
- **`FloatButton`**: A floating action button with additional properties.
- **`ToggleButton`**: A button that toggles between two states.
- **`DropdownButton`**: A button that shows a dropdown menu when clicked.

## Conclusion

The `Button` component is a fundamental UI element that provides a wide range of customization options and ensures accessibility. Use this component to create consistent and interactive buttons throughout your application.

For more information on other components and their usage, refer to the respective documentation files.
