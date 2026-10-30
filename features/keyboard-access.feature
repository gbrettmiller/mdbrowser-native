@keyboard-access
Feature: Keyboard access
  As a reader who doesn't use a mouse
  I want to follow links, scroll and change settings from the keyboard
  So that I can read the web of Markdown

  # These scenarios drive the rules the widgets use (stops, reach, scroll keys, the panel rule)
  # without a window. How the focus ring looks, Tab through the toolbar and Settings, the arrow
  # keys on a choice, and Esc returning to the page are checked by unit tests and screenshots.
  # Offsets come from the theme: a page is the window less three lines, a line is the body size
  # times the line spacing. Pages are set 600 pixels wide.

  Scenario: Tab walks the links in reading order
    Given a document "three.md" with 3 links
    And I am viewing "three.md"
    When I press "Tab"
    Then the focus is on link 1
    When I press "Tab"
    Then the focus is on link 2

  Scenario: Shift+Tab walks back
    Given a document "three.md" with 3 links
    And I am viewing "three.md"
    And the focus is on link 2
    When I press "Shift+Tab"
    Then the focus is on link 1

  Scenario: Tab past the last link leaves the page
    Given a document "three.md" with 3 links
    And I am viewing "three.md"
    And the focus is on link 3
    When I press "Tab"
    Then the focus is on the next control after the page

  Scenario: Shift+Tab before the first link leaves the page backwards
    Given a document "three.md" with 3 links
    And I am viewing "three.md"
    And the focus is on link 1
    When I press "Shift+Tab"
    Then the focus is on the previous control before the page

  Scenario: A page with no links is passed straight through
    Given a document "plain.md" with no links
    And I am viewing "plain.md"
    When I press "Tab"
    Then the focus is on the next control after the page

  Scenario: Enter follows the focused link
    Given a document "three.md" with 3 links
    And I am viewing "three.md"
    And the focus is on link 2
    When I press "Enter"
    Then I am still viewing "l2.md"

  Scenario: A wrapped link has one stop
    Given a document "wrapped.md" whose only link wraps
    And I am viewing "wrapped.md"
    When I press "Tab"
    Then the focus is on link 1
    And there is 1 link stop
    And the ring covers several lines

  Scenario: Focus scrolls the link into view
    Given a document "far.md" with a link at the top and another far below
    And I am viewing "far.md"
    And the focus is on link 1
    When I press "Tab"
    Then the focus is on link 2
    And link 2 is in view

  Scenario Outline: Scroll keys move the page
    Given a document "long.md" that is 80 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    When I press "<key>"
    Then the page is <by>
    Examples:
      | key      | by                 |
      | PageDown | scrolled one page down |
      | Space    | scrolled one page down |
      | Down     | scrolled one line down |
      | End      | at the end             |

  Scenario: PageUp, Shift+Space and Home scroll back up
    Given a document "long.md" that is 80 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    When I press "End"
    And I press "PageUp"
    Then the page is scrolled one page up from the end
    When I press "Shift+Space"
    Then the page is scrolled two pages up from the end
    When I press "Home"
    Then the page is at the top

  Scenario: Scrolling stops at the ends
    Given a document "long.md" that is 80 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    When I press "PageUp"
    Then the page is at the top
    When I press "End"
    And I press "PageDown"
    Then the page is at the end

  Scenario: Space in the address bar doesn't scroll the page
    Given a document "long.md" that is 80 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    When I press "PageDown"
    And the focus is on the address bar
    And I press "Space"
    Then the page is scrolled one page down

  Scenario: PageDown scrolls the page wherever the focus is
    Given a document "long.md" that is 80 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    And the focus is on the address bar
    When I press "PageDown"
    Then the page is scrolled one page down

  Scenario: The window has a minimum size
    Then the smallest window is 480 by 360

  Scenario: A second panel gives way when the page would be squeezed
    Given the window is 700 pixels wide
    And the History panel is open
    When I open the Settings panel
    Then the History panel is closed
    And the Settings panel is open

  Scenario: Two panels stay open when there is room
    Given the window is 1200 pixels wide
    And the History panel is open
    When I open the Settings panel
    Then both panels are open
