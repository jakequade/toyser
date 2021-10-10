# Toyser: Browser sandbox implementation

Following https://limpet.net/mbrubeck/2014/08/13/toy-layout-engine-3-css.html

## Noted improvements in the tutorial

### CSS

- multiple css selector declarations / matches.
- cascading stylesheets ([source](https://limpet.net/mbrubeck/2014/08/23/toy-layout-engine-4-style.html)): stylesheets have hierarchy, so some should merge-or-override subsequent stylesheets' styles. To implement this, suggested:
    - Track the origin of each style
    - sort declarations by origin and importance
- computed CSS values: Many CSS values in browsers are not statically within stylesheets, but are calculated by the browser. A simple example is `width: calc(50% - 10%)`
- inheritance: An element should inherit its parents styles when it does not have the same key for itself.
- the `style` html attribute: any html element can have a `style` attribute to add styles to it. The `specified_values` style function could check for this and append values to whatever other values it finds.