<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/beonux/images/logos/beonux.svg"
alt="BeonUX logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# Card Collection Component

The `CardCollection` component is a versatile UI element designed to handle collections of card components in a clean and consistent manner. This document provides a detailed overview of the properties, events, and usage of the `CardCollection` component.

## Table of Contents

1. [Overview](#overview)
2. [Properties](#properties)
3. [Events](#events)
4. [Usage](#usage)
5. [Customization](#customization)
6. [Examples](#examples)
7. [Related Components](#related-components)

## Overview

The `CardCollection` component inherits from `CardCollectionCore` and extends its functionality with theme-specific styling and additional properties. It is designed to be highly customizable and accessible, making it suitable for a variety of applications.

## Properties

### Common Properties

- **`cards`**: The collection of card components displayed within the CardCollection.
  - Type: `array<Card>`
  - Default: `[]`
  - Example: `cards: [Card { title: "Card 1" }, Card { title: "Card 2" }]`

- **`isSelectable`**: Determines if the cards in the collection are selectable.
  - Type: `bool`
  - Default: `false`
  - Example: `isSelectable: true`

- **`selectedCard`**: The currently selected card in the collection.
  - Type: `Card`
  - Default: `null`
  - Example: `selectedCard: cards[0]`

## Events

- **`cardSelected`**: This event is triggered when a card is selected.
  - Example:

    ```slint
    CardCollection {
        cards: [Card { title: "Card 1" }, Card { title: "Card 2" }];
        cardSelected => {
            // Handle card selection event
        }
    }
    ```

## Usage

To use the `CardCollection` component, import it and include it in your layout. Customize its properties to fit your design requirements.

### Basic Usage

```slint
import { CardCollection } from "card_collection/card_collection.slint";

CardCollection {
    cards: [Card { title: "Card 1" }, Card { title: "Card 2" }];
}
```

### Selectable Card Collection

```slint
CardCollection {
    cards: [Card { title: "Card 1" }, Card { title: "Card 2" }];
    isSelectable: true;
    cardSelected => {
        // Handle card selection event
    }
}
```

## Customization

The `CardCollection` component can be customized to match your application's theme and design language.

### Custom Card Properties

```slint
CardCollection {
    cards: [
        Card { title: "Custom Card 1", backgroundColor: Theme.palette.primary },
        Card { title: "Custom Card 2", backgroundColor: Theme.palette.secondary }
    ];
}
```

## Examples

Here are some examples demonstrating different configurations of the `CardCollection` component.

### Example 1: Basic Card Collection

```slint
CardCollection {
    cards: [
        Card { title: "Card 1", content: "Content for card 1" },
        Card { title: "Card 2", content: "Content for card 2" }
    ];
}
```

### Example 2: Selectable Card Collection

```slint
CardCollection {
    cards: [
        Card { title: "Card 1", content: "Content for card 1" },
        Card { title: "Card 2", content: "Content for card 2" }
    ];
    isSelectable: true;
}
```

## Related Components

- **`CardCollectionCore`**: The base component for card collections, providing common properties and functionality.
- **`Card`**: A single card component with additional properties.

## Conclusion

The `CardCollection` component is a fundamental UI element that provides a wide range of customization options and ensures accessibility. Use this component to create consistent and interactive card collections throughout your application.

For more information on other components and their usage, refer to the respective documentation files.
