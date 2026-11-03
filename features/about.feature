@about
Feature: About
  As a reader
  I want to see what this app is, its version and its licence
  So that I can report a problem, check what I am running and know what it sends

  # About is a built-in page (ADR-013): a page like any other, at about:mdbrowser,
  # with no History entry. Behaviour that is not about reaching it, such as its link
  # policy and the reader's look, is covered by unit tests.

  Background:
    Given a file "post.md" containing "# Post"
    And I am viewing "post.md"

  Scenario: Type the address
    When I enter "about:mdbrowser"
    Then I see the document titled "About mdBrowser"
    And the address bar shows "about:mdbrowser"

  Scenario: Bare about is the About page too
    When I enter "about:"
    Then I see the document titled "About mdBrowser"

  Scenario: An unknown about address is an error
    When I enter "about:nothing"
    Then the error shows the address "about:nothing"

  Scenario: About shows the version and platform
    When I enter "about:mdbrowser"
    Then the page shows the version from Cargo.toml
    And the page shows this platform

  Scenario: Open About from Settings
    Given the Settings panel is open
    When I choose About in Settings
    Then I see the document titled "About mdBrowser"
    And the address bar shows "about:mdbrowser"
    And the Settings panel is closed

  Scenario Outline: Open About from the keyboard
    When I press "<chord>"
    Then I see the document titled "About mdBrowser"
    Examples:
      | chord  |
      | F1     |
      | Ctrl+? |

  Scenario: F1 works from the address bar
    Given the focus is on the address bar
    When I press "F1"
    Then I see the document titled "About mdBrowser"

  Scenario: F1 on About does nothing more
    Given I pressed "F1"
    When I press "F1"
    And I go back
    Then I see the document titled "Post"

  Scenario: About is not in History, and Back and Forward work
    Given I pressed "F1"
    Then the History panel is closed
    And History does not list "About mdBrowser"
    When I go back
    Then I see the document titled "Post"
    When I go forward
    Then I see the document titled "About mdBrowser"

  Scenario: About starts no request
    When I press "F1"
    Then the site was not asked for "/"

  Scenario: Open About from the welcome page
    When the app starts with no address given
    Then I see the welcome page
    When I follow the welcome page's About link
    Then I see the document titled "About mdBrowser"
