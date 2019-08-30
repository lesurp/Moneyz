# TODO

1. Improve UI
2. Colors

Red / green for debit / credit, when balance for a budget or monthly is negative etc.

3. Refacto translations

Write a struct with getters for ts (potentially with arguments when formatting things e.g. money numbers etc), and one field. Then generate the field's type from the method names of the parent struct.
With this child struct, deserialize translation files, and return the parent struct whose methods call the child's to get the formatting etc.

4. Option to change ts (means adding a config.json)
5. Popup window to have the total balance over a given time interval
6. Handle decimal numbers (store things as integers, divide by, the usual)