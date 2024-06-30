<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg"
alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Divider Component

The `Divider` component is a versatile UI element that supports multiple styles. It is useful for creating visual separation between sections of your application.

## Features

- **Multiple Types**: The `Divider` component supports various types:
  - Horizontal
  - Vertical
  - Full-bleed
  - Inset
  - Centered
  - Image
  - Shadow

- **Alignment Options**: For horizontal dividers, you can set the alignment to left, center, or right.

- **Customizable Appearance**:
  - Thickness: Set the thickness of the divider.
  - Inset: Specify the inset value for the inset divider.
  - Shadow Color: Customize the shadow color for the shadow divider.
  - Image Source: Provide a source image for the image divider.

## Usage

To use the `Divider` component, import it and configure it as needed:

```slint
import { Divider } from "divider.slint";

// Example usage
export component Example inherits Rectangle {
    Divider {
        type: "horizontal"
        horizontal_alignment: "center"
        thickness: 2px
        inset: 20px
    }

    Divider {
        type: "vertical"
        thickness: 3px
    }

    Divider {
        type: "image"
        image: Images.some_image
        thickness: 4px
    }

    Divider {
        type: "shadow"
        shadow_color: Theme.palette.gray.darker(50%)
        thickness: 5px
    }
}
```

### Properties

- **type**: Specifies the type of the divider. Options include "horizontal", "vertical", "full-bleed", "inset", "centered", "image", and "shadow".
- **horizontal_alignment**: Sets the alignment of the horizontal divider. Options are "left", "center", and "right".
- **image**: The source image for the image divider.
- **shadow_color**: The shadow color for the shadow divider.
- **thickness**: The thickness of the divider.
- **inset**: The inset value for the inset divider.

## Installation

To install and use the `Divider` component, make sure to include it in your project dependencies and import it as shown in the usage example.