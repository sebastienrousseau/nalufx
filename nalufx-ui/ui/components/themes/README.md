<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg" alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Slint Schemes

The Slint schemes provided include various configurations for color, duration, font, screen, and space. These schemes are designed to provide a consistent and customizable styling foundation for your Slint applications. This document provides an overview of each scheme and how to use them.

## Table of Contents

1. [Overview](#overview)
2. [Color Scheme](#color-scheme)
3. [Duration Scheme](#duration-scheme)
4. [Font Scheme](#font-scheme)
5. [Screen Scheme](#screen-scheme)
6. [Space Scheme](#space-scheme)
7. [Usage](#usage)
8. [Examples](#examples)

## Overview

Each scheme file defines a set of related properties that can be used throughout your Slint application to ensure a cohesive and maintainable design system. These schemes cover a wide range of design aspects such as colors, typography, timing, layout spacing, and responsive screen sizes.

## Color Scheme

The `color_scheme.slint` file defines a palette of colors used throughout the application.

### Properties

- `primary-color`: The main color used for primary actions and highlights.
- `secondary-color`: A secondary color for less prominent elements.
- `background-color`: The default background color.
- `text-color`: The primary text color.

### Example

```slint
import { color_scheme } from "schemes/color_scheme.slint";

color_scheme {
    primary-color: #3498db;
    secondary-color: #2ecc71;
    background-color: #ecf0f1;
    text-color: #2c3e50;
}
```

## Duration Scheme

The `duration_scheme.slint` file defines timing values for animations and transitions.

### Properties

- `short-duration`: A short duration, typically used for quick transitions.
- `medium-duration`: A medium duration for standard animations.
- `long-duration`: A longer duration for more significant transitions.

### Example

```slint
import { duration_scheme } from "schemes/duration_scheme.slint";

duration_scheme {
    short-duration: 200ms;
    medium-duration: 400ms;
    long-duration: 600ms;
}
```

## Font Scheme

The `font_scheme.slint` file defines the typography settings for the application.

### Properties

- `font-family`: The primary font family.
- `font-size`: Default font size.
- `font-weight`: Default font weight.
- `line-height`: Default line height.

### Example

```slint
import { font_scheme } from "schemes/font_scheme.slint";

font_scheme {
    font-family: "Roboto";
    font-size: 16px;
    font-weight: 400;
    line-height: 1.5;
}
```

## Screen Scheme

The `screen_scheme.slint` file defines breakpoints for responsive design.

### Properties

- `small-screen`: Maximum width for small screens.
- `medium-screen`: Maximum width for medium screens.
- `large-screen`: Minimum width for large screens.

### Example

```slint
import { screen_scheme } from "schemes/screen_scheme.slint";

screen_scheme {
    small-screen: 600px;
    medium-screen: 960px;
    large-screen: 1280px;
}
```

## Space Scheme

The `space_scheme.slint` file defines spacing values for layout and design.

### Properties

- `small-space`: Small spacing value.
- `medium-space`: Medium spacing value.
- `large-space`: Large spacing value.

### Example

```slint
import { space_scheme } from "schemes/space_scheme.slint";

space_scheme {
    small-space: 8px;
    medium-space: 16px;
    large-space: 32px;
}
```

## Usage

To use these schemes in your Slint application, import the desired scheme file and apply its properties to your components.

### Basic Usage

```slint
import { color_scheme } from "schemes/color_scheme.slint";
import { font_scheme } from "schemes/font_scheme.slint";

component MyApp {
    background: color_scheme.background-color;
    font-family: font_scheme.font-family;
    font-size: font_scheme.font-size;
}
```

## Examples

### Example 1: Applying Color and Font Schemes

```slint
import { color_scheme } from "schemes/color_scheme.slint";
import { font_scheme } from "schemes/font_scheme.slint";

component ExampleComponent {
    background: color_scheme.primary-color;
    color: color_scheme.text-color;
    font-family: font_scheme.font-family;
    font-size: font_scheme.font-size;
}
```

### Example 2: Responsive Design with Screen Scheme

```slint
import { screen_scheme } from "schemes/screen_scheme.slint";

component ResponsiveComponent {
    @media (max-width: screen_scheme.small-screen) {
        // Styles for small screens
    }
    @media (min-width: screen_scheme.medium-screen) {
        // Styles for medium screens
    }
    @media (min-width: screen_scheme.large-screen) {
        // Styles for large screens
    }
}
```

## Conclusion

These Slint schemes provide a robust foundation for building consistent and maintainable UI components. Utilize these schemes to streamline your design process and ensure a cohesive user experience.

For more information on other components and their usage, refer to the respective documentation files.
