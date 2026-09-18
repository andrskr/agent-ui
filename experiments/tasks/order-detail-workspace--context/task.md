Build an order detail workspace with a master list and a separate detail panel.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Use the three orders in `references/data.json`. Copy the data into `src/` before using it. Preserve
the supplied customers, addresses, amounts, order dates, and event history. Display money in USD.
Amounts in the data are integer cents. Use the supplied fixed action timestamp for new events.

Show a page title and a compact order list beside the selected order's details. Each list item shows
order number, customer, date, status, and total. Select the first order initially. Make the selected
item visually distinct. Do not use a large table for the master list.

The detail header shows order number, status, date, and customer. Below it, organize line items,
payment summary, delivery address, and order history into clear sections. Line items show product,
quantity, unit price, and line total. Calculate subtotal, discount, shipping, tax, and final total
from the supplied data. The master list total and detail total must agree.

Show order history as a chronological timeline with the newest event first. Each event includes its
title, timestamp, and description. Long addresses, product names, and event descriptions must remain
readable within their sections.

Selecting another order updates all detail sections together. Preserve local changes to an order
when the user selects another order and returns.

The only status action is Mark as shipped. Show it only for Ready to ship orders. It opens a
confirmation dialog with the order number and delivery address. Cancel makes no changes. Confirm
shows a saving state for one second. During saving, prevent another submission and disable order
selection and dialog dismissal.

After saving, close the dialog, change that order to Shipped, add an Order shipped timeline event
with the fixed action timestamp, and show a confirmation message. Update the status in both the
master list and detail header. Remove the action so the same order cannot ship twice. Orders that
start as Processing or Shipped do not offer this action.

Keep all state local. Do not add search, filters, order creation, payment actions, or shipment
tracking. Do not call a network service or preserve changes after a page reload. Do not add
dependencies or change files outside `src/`. In your final response, state what you built and how
you checked it.
