@settings
Feature: Settings and home page
  As a reader
  I want to choose how pages look and where I start
  So that the browser opens comfortably, every time

  # Appearance names are as the Settings panel shows them. Following the
  # operating system's light or dark mode is decided in the UI, so it is covered
  # by unit tests and screenshots, not here.

  Scenario: Appearance follows the system by default
    Given I am viewing "index.md"
    Then the appearance is "System"

  Scenario Outline: Choosing an appearance keeps the reader on the page
    Given I am viewing "index.md"
    When I choose the appearance "<appearance>"
    Then the appearance is "<appearance>"
    And I am still viewing "index.md"
    Examples:
      | appearance |
      | Light      |
      | Dark       |
      | System     |

  Scenario: The appearance survives a restart
    Given I chose the appearance "Dark"
    When I restart the app
    Then the appearance is "Dark"

  Scenario: An unknown stored appearance falls back to the system
    Given the stored appearance is "sepia"
    When I restart the app
    Then the appearance is "System"

  Scenario: Settings saved before appearance and home existed still load
    Given the stored settings are "{\"font\":\"inter\"}"
    When I restart the app
    Then the reading font is "Inter"
    And the appearance is "System"
    And no home page is set

  Scenario: Home opens the welcome page when no home page is set
    Given I am viewing "index.md"
    When I press Home
    Then I see the welcome page
    And back is enabled

  Scenario: The welcome page is not recorded in History
    When I press Home
    Then the welcome page is not in History

  Scenario: Home opens the home page
    Given I set the home page to the absolute path of "notes/a.md"
    And I am viewing "index.md"
    When I press Home
    Then I am still viewing "notes/a.md"

  Scenario: Back from Home returns to where the reader was
    Given I set the home page to the absolute path of "notes/a.md"
    And I am viewing "index.md"
    When I press Home
    And I go back
    Then I am still viewing "index.md"

  Scenario: The home page is saved in its canonical form
    When I set the home page to the absolute path of "notes/a.md"
    Then the home page is the absolute path of "notes/a.md"

  Scenario: The home page survives a restart
    Given I set the home page to the absolute path of "notes/a.md"
    When I restart the app
    Then the home page is the absolute path of "notes/a.md"

  Scenario Outline: A home page that isn't an address is refused
    Given I set the home page to the absolute path of "notes/a.md"
    When I set the home page to "<value>"
    Then I am told "That isn't a path or web address of a Markdown file, so the home page is unchanged."
    And the home page is the absolute path of "notes/a.md"
    Examples:
      | value                       |
      | notes/a.md                  |
      | ftp://example.com/readme.md |

  Scenario: Clearing the home page goes back to the welcome page
    Given I set the home page to the absolute path of "notes/a.md"
    When I clear the home page
    Then no home page is set
    When I press Home
    Then I see the welcome page

  Scenario: The app opens the home page on launch
    Given I set the home page to the absolute path of "notes/a.md"
    When the app starts with no address given
    Then I am still viewing "notes/a.md"

  Scenario: The app opens the welcome page on launch with no home page
    When the app starts with no address given
    Then I see the welcome page
