Build a product gallery using the installed Astryx components.

Show the six products in `references/products.json` in a consistent grid. Each item has a local
placeholder image, name, category, price in USD, and an optional New or Sold out badge. Use the
supplied SVG images in `references/images/`. Copy them under `src/` to use them in the app.

Above the grid, add a category filter and a sort control for name or price in either direction.
Start with all categories, name ascending, and no selection. Categories are Bags, Drinkware, and
Stationery. Show an empty state if no products match the filter.

Let users select one available product. Highlight its selection and show its name and price in a
summary below the grid. Before selection, show “Select a product.” Sold-out products cannot be
selected. Keep the selection when filtering or sorting the grid.

Build only the gallery, controls, and selection summary. Use the fixed sample data and local state.
Do not add product detail pages, checkout, network calls, or persistence.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
