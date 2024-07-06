<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg"
alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Slider Component

The `Slider` component is a customizable UI element designed for selecting values within a specified range. This document provides a detailed overview of the properties, events, and usage of the `Slider` component.

## Table of Contents

1. [Overview](#overview)
2. [Properties](#properties)
3. [Events](#events)
4. [Usage](#usage)
5. [Customization](#customization)
6. [Examples](#examples)
7. [Conclusion](#conclusion)

## Overview

The `Slider` component inherits from `Rectangle` and extends its functionality with theme-specific styling and additional properties. It is designed to be highly customizable and accessible, making it suitable for a variety of applications.

## Properties

### Common Properties

- **`min-value`**: The minimum value of the slider range.
  - Type: `float`
  - Default: `0.0`
  - Example: `min-value: 0.0`

- **`max-value`**: The maximum value of the slider range.
  - Type: `float`
  - Default: `100.0`
  - Example: `max-value: 200.0`

- **`value`**: The current value of the slider.
  - Type: `float`
  - Default: `50.0`
  - Example: `value: 75.0`

- **`step`**: The step size for value changes.
  - Type: `float`
  - Default: `1.0`
  - Example: `step: 0.5`

## Events

- **`pointer-event`**: This event is triggered when a pointer event occurs on the slider's handle or track.
  - Example:

    ```slint
    Slider {
        min-value: 0.0;
        max-value: 100.0;
        value: 50.0;
        pointer-event => {
            // Handle pointer event
        }
    }
    ```

## Usage

To use the `Slider` component, import it and include it in your layout. Customize its properties to fit your design requirements.

### Basic Usage

```slint
import { Slider } from "slider/slider.slint";

Slider {
    min-value: 0.0;
    max-value: 100.0;
    value: 50.0;
}
```

### Custom Range Slider

```slint
Slider {
    min-value: 10.0;
    max-value: 200.0;
    value: 50.0;
    step: 5.0;
}
```

## Customization

The `Slider` component can be customized to match your application's theme and design language.

### Custom Track Color

```slint
Slider {
    min-value: 0.0;
    max-value: 100.0;
    value: 50.0;
    // Custom track color can be defined in the theme
}
```

### Custom Handle Size

```slint
Slider {
    min-value: 0.0;
    max-value: 100.0;
    value: 50.0;
    handle.width: 30px;
    handle.height: 30px;
}
```

## Examples

Here are some examples demonstrating different configurations of the `Slider` component.

### Example 1: Default Slider

```slint
Slider {
    min-value: 0.0;
    max-value: 100.0;
    value: 50.0;
}
```

### Example 2: Slider with Custom Range

```slint
Slider {
    min-value: 0.0;
    max-value: 200.0;
    value: 75.0;
    step: 10.0;
}
```

## Conclusion

The `Slider` component is a fundamental UI element that provides a wide range of customization options and ensures accessibility. Use this component to create interactive sliders for value selection in your application.

For more information on other components and their usage, refer to the respective documentation files.
