<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg"
alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Carousel Component

The `Carousel` component is a versatile UI element designed to display a series of images in a linear or rotary fashion. This document provides a detailed overview of the properties, events, and usage of the `Carousel` component.

## Table of Contents

1. [Overview](#overview)
2. [Properties](#properties)
3. [Events](#events)
4. [Usage](#usage)
5. [Customization](#customization)
6. [Examples](#examples)
7. [Related Components](#related-components)

## Overview

The `Carousel` component allows users to browse through a series of images, either linearly or in a circular manner. It is designed to be highly customizable, making it suitable for various applications in Slint.

## Properties

### Common Properties

- **`sources`**: An array of image sources to display in the carousel.
- **`type`**: The type of carousel behavior (`Linear` or `Rotary`). Default is `Linear`.
- **`fold-stretch`**: A float property that defines the stretch factor for non-active items. Default is `1.0`.
- **`fold-width`**: The width of non-active items. Default is `100px`.
- **`fold-height`**: The height of the carousel. Default is the height of the root component.
- **`focus-main`**: A boolean property to determine whether to focus on the main (active) item. Default is `true`.
- **`current-index`**: The index of the currently active item.

## Events

### Common Events

- **`item-clicked`**: This event is triggered when an item in the carousel is clicked.

## Usage

To use the `Carousel` component in your Slint application, import it and include it in your UI code. Below is an example of how to use the `Carousel` component.

### Basic Carousel

```slint
import { Carousel } from "components/carousel.slint";

Carousel {
    sources: [
        "image1.png",
        "image2.png",
        "image3.png"
    ];
    type: CarouselType.Linear;
    item-clicked => {
        // Handle item click event
    }
}
```

## Customization

The `Carousel` component can be customized to match your application’s theme and design language.

### Custom Fold Width and Height

You can customize the width and height of non-active items in the carousel.

```slint
Carousel {
    sources: [
        "image1.png",
        "image2.png",
        "image3.png"
    ];
    fold-width: 120px;
    fold-height: 150px;
}
```

### Custom Colors and Focus

You can customize the focus behavior and the stretch factor for non-active items.

```slint
Carousel {
    sources: [
        "image1.png",
        "image2.png",
        "image3.png"
    ];
    focus-main: false;
    fold-stretch: 0.8;
}
```

## Examples

Here are some examples demonstrating different configurations of the `Carousel` component.

### Example 1: Default Linear Carousel

```slint
Carousel {
    sources: [
        "image1.png",
        "image2.png",
        "image3.png"
    ];
}
```

### Example 2: Rotary Carousel

```slint
Carousel {
    sources: [
        "image1.png",
        "image2.png",
        "image3.png"
    ];
    type: CarouselType.Rotary;
}
```

### Example 3: Custom Styled Carousel

```slint
Carousel {
    sources: [
        "image1.png",
        "image2.png",
        "image3.png"
    ];
    fold-width: 120px;
    fold-height: 150px;
    focus-main: false;
    fold-stretch: 0.8;
}
```

## Conclusion

The `Carousel` component is a fundamental UI element that provides a wide range of customization options. Use this component to create engaging and interactive carousels in your application.

For more information on other components and their usage, refer to the respective documentation files.
