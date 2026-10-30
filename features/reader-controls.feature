@reader-controls
Feature: Reader controls
  As a reader
  I want to set the size, spacing and colours of the text
  So that documents are comfortable for my eyes

  # Size and spacing names are as the Settings panel shows them.

  Scenario: Documents open at the default size and spacing
    Given I am viewing "index.md"
    Then the text size is "18"
    And the spacing is "Comfortable"

  Scenario: Larger text re-sets the open document
    Given I am viewing "index.md"
    When I make the text larger
    Then the document is set at a larger size
    And I am still viewing "index.md"

  Scenario: Text size stops at its limits
    Given I am viewing "index.md"
    When I make the text larger 40 times
    And I make the text larger
    Then the text size is the largest step
    When I make the text smaller 80 times
    Then the text size is the smallest step

  Scenario: Resetting the size returns to the default
    Given I made the text larger
    When I reset the text size
    Then the text size is "18"

  Scenario Outline: Choosing a spacing re-sets the open document
    Given I am viewing "index.md"
    When I choose the spacing "<spacing>"
    Then the spacing is "<spacing>"
    And I am still viewing "index.md"
    Examples:
      | spacing     |
      | Compact     |
      | Comfortable |
      | Spacious    |

  Scenario: Compact is shorter than Comfortable and Spacious is taller
    Given I am viewing "index.md"
    When I choose the spacing "Compact"
    Then the page is shorter than at "Comfortable"
    When I choose the spacing "Spacious"
    Then the page is taller than at "Comfortable"

  Scenario: Reset restores size and spacing only
    Given I chose the reading font "Inter"
    And I made the text larger
    And I chose the spacing "Spacious"
    When I reset the reader controls
    Then the text size is "18"
    And the spacing is "Comfortable"
    And the reading font is "Inter"

  Scenario: Custom colours start from the current palette
    Given I chose the appearance "Dark"
    When I choose the appearance "Custom"
    Then the custom page colour is the dark page colour

  Scenario Outline: A custom colour is accepted as hex
    Given I chose the appearance "Custom"
    When I set the custom "<which>" colour to "<value>"
    Then the custom "<which>" colour is "<value>"
    Examples:
      | which | value   |
      | Page  | #fdf6e3 |
      | Text  | #333    |
      | Link  | #0b5fa5 |

  Scenario Outline: A value that isn't a colour is refused
    Given I chose the appearance "Custom"
    When I set the custom "Text" colour to "<value>"
    Then I am told "That isn't a colour. Use a hex value such as #222 or #fdf6e3."
    And the custom "Text" colour is what it was
    Examples:
      | value |
      | red   |
      | #12   |
      | #ggg  |
      |       |

  Scenario: Text that can't be read on the page is refused
    Given I chose the appearance "Custom"
    And I set the custom "Page" colour to "#ffffff"
    When I set the custom "Text" colour to "#eeeeee"
    Then I am told that the text colour is too close to the page colour
    And the custom "Text" colour is what it was

  Scenario: Links that can't be read on the page are refused
    Given I chose the appearance "Custom"
    And I set the custom "Page" colour to "#ffffff"
    When I set the custom "Link" colour to "#ffff00"
    Then I am told that the link colour is too close to the page colour
    And the custom "Link" colour is what it was

  Scenario: Custom colours survive a restart
    Given I chose the appearance "Custom"
    And I set the custom "Page" colour to "#fdf6e3"
    When I restart the app
    Then the appearance is "Custom"
    And the custom "Page" colour is "#fdf6e3"

  Scenario: Choosing another appearance keeps the custom colours
    Given I chose the appearance "Custom"
    And I set the custom "Page" colour to "#fdf6e3"
    When I choose the appearance "Light"
    And I choose the appearance "Custom"
    Then the custom "Page" colour is "#fdf6e3"

  Scenario: Settings saved before reader controls existed still load
    Given the stored settings are "{\"font\":\"inter\",\"appearance\":\"dark\"}"
    When I restart the app
    Then the text size is "18"
    And the spacing is "Comfortable"
    And the appearance is "Dark"

  Scenario: A bad stored value doesn't discard its good neighbours
    Given the stored settings are "{\"font\":\"inter\",\"spacing\":\"tight\",\"appearance\":\"dark\"}"
    When I restart the app
    Then the reading font is "Inter"
    And the appearance is "Dark"
    And the spacing is "Comfortable"

  Scenario Outline: Unknown stored values fall back to the defaults
    Given the stored settings are "<stored>"
    When I restart the app
    Then the text size is "18"
    And the spacing is "Comfortable"
    And the custom "Page" colour is "#fbfaf7"
    Examples:
      | stored                                                             |
      | {\\"spacing\\":\\"tight\\"}                                        |
      | {\\"text_size\\":999}                                              |
      | {\\"text_size\\":\\"big\\"}                                        |

  Scenario: A stored custom set that can't be used falls back to the light colours
    Given the stored settings are "{\"appearance\":\"custom\",\"custom\":{\"ink\":\"red\"}}"
    When I restart the app
    Then the appearance is "Custom"
    And the custom "Page" colour is "#fbfaf7"
