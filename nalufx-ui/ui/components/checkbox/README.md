<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg"
alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# CheckBox Component

The `CheckBox` component is a versatile UI element designed to handle user interactions in a clean and consistent manner. This document provides a detailed overview of the properties, events, and usage of the `CheckBox` component.

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

The `CheckBox` component allows users to make a binary choice, such as a yes/no or on/off selection. It is designed to be highly customizable and accessible, making it suitable for a variety of applications in Slint.

## Properties

### Common Properties

- **`checked`**: A boolean property that determines whether the checkbox is checked.

## States

- **`checked`**: When the `checked` property is `true`, the checkbox appears filled.

## Events

### Common Events

- **`clicked`**: This event is triggered when the checkbox is clicked.

## Usage

To use the `CheckBox` component in your Slint application, import it and include it in your UI code. Below is an example of how to use the `CheckBox` component.

### Basic CheckBox

```slint
import { CheckBox } from "components/checkbox.slint";

CheckBox {
    checked: true;
    clicked => {
        // Handle click event
    }
}
```

## Customization

The `CheckBox` component can be customized to match your application’s theme and design language.

### Custom Colors

You can customize the colors of the checkbox using the `Theme` properties.

```slint
CheckBox {
    checked: true;
    i-container {
        border-color: NalufxTheme.palette.system-indigo;
        background: Theme.palette.purple;
    }
    i-check-icon {
        colorize: Theme.palette.pure-black;
    }
}
```

## Examples

Here are some examples demonstrating different configurations of the `CheckBox` component.

### Example 1: Default CheckBox

```slint
CheckBox {
    checked: true;
}
```

### Example 2: Unchecked CheckBox

```slint
CheckBox {
    checked: false;
}
```

### Example 3: Custom Styled CheckBox

```slint
CheckBox {
    checked: true;
    i-container {
        border-color: Theme.palette.primary;
        background: Theme.palette.secondary;
    }
    i-check-icon {
        colorize: Theme.palette.accent;
    }
}
```

## Related Components

- **`RadioButton`**: A radio button component for selecting one option from a set.
- **`ToggleButton`**: A button that toggles between two states.
- **`Switch`**: A switch component for toggling between on and off states.

## Conclusion

The `CheckBox` component is a fundamental UI element that provides a wide range of customization options and ensures accessibility. Use this component to create consistent and interactive checkboxes throughout your application.

For more information on other components and their usage, refer to the respective documentation files.