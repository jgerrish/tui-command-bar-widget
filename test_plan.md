# tui-command-bar-widget Test Plan #

This document outlines a test plan for the tui-command-bar-widget project.

Right now it mostly focuses on test cases, and not how to execute the
plan.

## Event Testing ##

Test names should be refactored to be consistent.

Test cases to consider:

- normal mode
  - key is unregistered
    - event read error results in sane result
    - key event is passed to parent as unhandled
    - non-key event is passed to parent as unhandled
  - key is registered
    - event read error results in sane result
    - unmatched key is passed to parent as unhandled
    - matched key is handled
    - non-key event is passed to parent as unhandled
  - editing mode
    - test that key is registered
    - event read error results in sane result
    - escape key results in change of mode
    - other key input is captured
    - other event is passed to parent as unhandled

## UI Testing ##

The command bar by default should expand to fill it's container.

There should be one line for editing text.

It should receive focus when the command key is pressed, and leave
focus when the escape key is pressed.

Editing shouldn't extend beyond the length of the widget.
