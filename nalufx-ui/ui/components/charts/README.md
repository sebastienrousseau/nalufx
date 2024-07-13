<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg"
alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Chart and Tile Components

This repository contains several versatile UI components designed for handling various types of charts and tiles in a clean and consistent manner. This document provides a detailed overview of the properties, events, and usage of each component.

## Table of Contents

1. [Overview](#overview)
2. [Components](#components)
   - [Balance Chart](#balance-chart)
   - [Bar Chart](#bar-chart)
   - [Bar Tiles](#bar-tiles)
   - [Chart Axis](#chart-axis)
   - [Chart Pattern](#chart-pattern)
   - [Tile](#tile)
3. [Usage](#usage)
4. [Customization](#customization)
5. [Examples](#examples)
6. [Related Components](#related-components)

## Overview

The chart and tile components in this repository are designed to be highly customizable and accessible, making them suitable for a variety of applications. Each component has specific properties and events that allow for detailed control and interaction.

## Components

### Balance Chart

The `BalanceChart` component is designed to display balance-related data in a graphical format.

#### Properties

- **`data`**: The data to be displayed in the balance chart.
  - Type: `array<BalanceData>`
  - Default: `[]`
  - Example: `data: [BalanceData { value: 100 }, BalanceData { value: 200 }]`

#### Events

- **`dataUpdated`**: Triggered when the balance data is updated.

    ```slint
    BalanceChart {
        data: [BalanceData { value: 100 }, BalanceData { value: 200 }];
        dataUpdated => {
            // Handle data updated event
        }
    }
    ```

### Bar Chart

The `BarChart` component is used to display data in a bar chart format.

#### Properties

- **`data`**: The data to be displayed in the bar chart.
  - Type: `array<BarData>`
  - Default: `[]`
  - Example: `data: [BarData { value: 50 }, BarData { value: 150 }]`

#### Events

- **`barSelected`**: Triggered when a bar is selected.

    ```slint
    BarChart {
        data: [BarData { value: 50 }, BarData { value: 150 }];
        barSelected => {
            // Handle bar selection event
        }
    }
    ```

### Bar Tiles

The `BarTiles` component combines tiles and bar charts for a unique display format.

#### Properties

- **`tiles`**: The collection of tiles within the BarTiles component.
  - Type: `array<Tile>`
  - Default: `[]`
  - Example: `tiles: [Tile { title: "Tile 1" }, Tile { title: "Tile 2" }]`

#### Events

- **`tileClicked`**: Triggered when a tile is clicked.

    ```slint
    BarTiles {
        tiles: [Tile { title: "Tile 1" }, Tile { title: "Tile 2" }];
        tileClicked => {
            // Handle tile click event
        }
    }
    ```

### Chart Axis

The `ChartAxis` component is used to define the axes of a chart.

#### Properties

- **`axisType`**: The type of axis (e.g., `x`, `y`).
  - Type: `string`
  - Default: `x`
  - Example: `axisType: "y"`

#### Events

- **`axisChanged`**: Triggered when the axis type is changed.

    ```slint
    ChartAxis {
        axisType: "y";
        axisChanged => {
            // Handle axis change event
        }
    }
    ```

### Chart Pattern

The `ChartPattern` component allows for the definition of patterns within a chart.

#### Properties

- **`patternType`**: The type of pattern (e.g., `stripes`, `dots`).
  - Type: `string`
  - Default: `stripes`
  - Example: `patternType: "dots"`

#### Events

- **`patternUpdated`**: Triggered when the pattern type is updated.

    ```slint
    ChartPattern {
        patternType: "dots";
        patternUpdated => {
            // Handle pattern update event
        }
    }
    ```

### Tile

The `Tile` component is a basic unit used within other components.

#### Properties

- **`title`**: The title of the tile.
  - Type: `string`
  - Default: `""`
  - Example: `title: "Sample Tile"`

#### Events

- **`tileSelected`**: Triggered when the tile is selected.

    ```slint
    Tile {
        title: "Sample Tile";
        tileSelected => {
            // Handle tile selection event
        }
    }
    ```

## Usage

To use any of these components, import them and include them in your layout. Customize their properties to fit your design requirements.

### Basic Usage

```slint
import { BalanceChart } from "balance_chart.slint";
import { BarChart } from "bar_chart.slint";

BalanceChart {
    data: [BalanceData { value: 100 }, BalanceData { value: 200 }];
}

BarChart {
    data: [BarData { value: 50 }, BarData { value: 150 }];
}
```

## Customization

The components can be customized to match your application's theme and design language.

### Custom Properties

```slint
BarChart {
    data: [
        BarData { value: 50, color: Theme.palette.primary },
        BarData { value: 150, color: Theme.palette.secondary }
    ];
}
```

## Examples

Here are some examples demonstrating different configurations of the components.

### Example 1: Basic Balance Chart

```slint
BalanceChart {
    data: [
        BalanceData { value: 100, description: "January" },
        BalanceData { value: 200, description: "February" }
    ];
}
```

### Example 2: Interactive Bar Chart

```slint
BarChart {
    data: [
        BarData { value: 50, label: "Q1" },
        BarData { value: 150, label: "Q2" }
    ];
    barSelected => {
        // Handle bar selection event
    }
}
```

## Related Components

- **`BalanceChartCore`**: The base component for balance charts, providing common properties and functionality.
- **`TileCore`**: The base component for tiles.

## Conclusion

The chart and tile components provided in this repository are essential UI elements that offer a wide range of customization options and ensure accessibility. Use these components to create consistent and interactive chart and tile layouts throughout your application.

For more information on other components and their usage, refer to the respective documentation files.
