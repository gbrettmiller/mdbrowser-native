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
