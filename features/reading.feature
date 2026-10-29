@reading
Feature: Choose a reading font
  As a reader
  I want to pick the typeface documents are set in
  So that long documents are comfortable to read

  # Font names are as the font menu shows them. "Set in" checks the laid-out
  # page, so a font that silently fell back to another family would fail.

  Scenario: Literata is the default
    Given I am viewing "index.md"
    Then the reading font is "Literata"
    And the document is set in "Literata"

  Scenario Outline: Choosing a font re-sets the open document
    Given I am viewing "index.md"
    When I choose the reading font "<font>"
    Then the document is set in "<font>"
    And I am still viewing "index.md"
    Examples:
      | font              |
      | Source Serif 4    |
      | iA Writer Quattro |
      | Inter             |

  Scenario: The choice survives a restart
    Given I chose the reading font "Inter"
    When I restart the app
    Then the reading font is "Inter"

  Scenario: An unknown stored font falls back to the default
    Given the stored reading font is "comic-sans"
    When I restart the app
    Then the reading font is "Literata"
