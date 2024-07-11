<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg"
alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Popup Component

The `Popup` component is a versatile UI element designed to display overlay content in a modal-like fashion, using a card layout. This document provides a detailed overview of the properties, events, and usage of the `Popup` component.

## Table of Contents

1. [Overview](#overview)
2. [Properties](#properties)
3. [Events](#events)
4. [Usage](#usage)
5. [Customization](#customization)
6. [Examples](#examples)

## Overview

The `Popup` component is an extension of the `Card` component. It allows for displaying a customizable popup window with content that can dynamically appear or disappear based on the `is-open` state. This makes it ideal for modal dialogs, notifications, or any contextual overlay within an application.

## Properties

### Common Properties

- **`is-open`**: A boolean property that controls the visibility of the popup. Default is `false`.
- **`width`**, **`height`**: Dimensions of the popup, defaulting to `300px` by `300px`.
- **`background-color`**: Background color of the popup, inheriting from `NalufxTheme.palette.background`.
- **`corner-radius`**: Corner radius of the popup window, set to `15px`.
- **`opacity`**: Controls the opacity of the popup, with animations for visibility changes.

## Events

### Common Events

- **`close()`**: Callback triggered when the popup is requested to close, either by interaction or programmatically.

## Usage

To use the `Popup` component in your Slint application, import it and incorporate it within your UI hierarchy. Below is a basic example of how to integrate the `Popup` component.

### Basic Popup

```slint
import { Popup } from "components/popup.slint";

Popup {
    is-open: true;
    close => {
        // Actions to perform on close
    }
}
```

## Customization

The `Popup` component can be customized in various ways to fit the design language of your application.

### Customizing Dimensions and Style

You can adjust the dimensions, background color, and corner radius of the popup.

```slint
Popup {
    width: 400px;
    height: 200px;
    background-color: #FFFFFF;  // Assuming NalufxTheme has a white color defined
    corner-radius: 10px;
}
```

## Examples

Here are some examples demonstrating different configurations of the `Popup` component.

### Example 1: Default Popup

```slint
Popup {
    is-open: true;
}
```

### Example 2: Custom Styled Popup

```slint
Popup {
    width: 450px;
    height: 300px;
    background-color: #F0F0F0;  // Light grey background
    corner-radius: 20px;
    is-open: true;
}
```

## Conclusion

The `Popup` component is a fundamental UI element that provides a modal functionality with extensive customization options. Use this component to create engaging and contextually relevant overlays in your application.

For more information on other components and their usage, refer to the respective documentation files.

