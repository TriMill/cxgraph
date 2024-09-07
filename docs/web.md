# CXGraph Web UI

## Source

Enter the program to plot into the text area. For more information, see [the language docs](language.md).

The Graph button compiles the program and redraws the screen. This must be pressed after changes are made to the program to see them. This can also be accomplished by pressing `Shift`+`Enter` in the text area.

The Redraw button redraws the screen. If Auto Redraw is enabled, the screen will be redrawn automatically after every change (eg. dragging, zooming, changing options or variables).

## The plot

Most of the screen is occupied by the plot. Click and drag with the mouse to move around, and use the scroll wheel to zoom in and out.

## Options

Reset View resets the plot's position and scale. Help opens the documentation.

The Resolution slider controls the canvas's resolution scale on a range from x0.25 to x4. This is set to x1 by default. Higher values provide better visuals at the expense of performance.

Shading Intensity controls how intense the black and white shading near zero/infinity is. Setting this to zero disables shading.

Contours can be toggled with the contour checkboxes. Real and imaginary contours show the integer grid, argument contours show angles around the origin divided into 16 segments, and magnitude contours show magnitudes delineated by powers of two.

Standard coloring directly maps argument to hue in HSV while keeping saturation and value constant. Uniform coloring uses a modified mapping that tries to avoid variation in perceptual brightness. None disables coloring entirely.

Draw Axes and Unit Circle enable or disable the axes and circle overlays.

## Variables

Sliders can be added with the "+slider" button. Once the slider has been named, the variable name can be used in the program and the plot will redraw automatically when the slider's value is changed. The slider's start, step, and end default to 1, 0.01, and -1, respectively. The slider's value can also be edited directly.

Points can be added with the "+point" button. They behave similarly to sliders, but add a draggable point to the plot. The position of the point can also be edited directly.
