@source-view
Feature: View the Markdown source
  As a reader
  I want to see the Markdown a page is made of
  So that I can tell what the author wrote and what the browser drew

  # The source view is a lens on the page: not a History entry, not a setting.

  Scenario: The source shows the file as written, front matter included
    Given a file "front.md" containing "---\ntitle: Front\nauthor: Me\n---\n# Body\n"
    And I am viewing "front.md"
    When I view the source
    Then I see the source view
    And the source starts with the line "---"
    And the source is the text of "front.md"

  Scenario: Toggling again returns to the page
    Given I am viewing "index.md"
    And I viewed the source
    When I view the page
    Then I see the rendered page

  Scenario: A page without front matter shows none
    Given a file "plain.md" containing "# Plain\n\nJust text.\n"
    And I am viewing "plain.md"
    When I view the source
    Then the source is the text of "plain.md"
    And the source has no line "---"

  Scenario: An empty file shows an empty source
    Given a file "empty.md" containing ""
    And I am viewing "empty.md"
    When I view the source
    Then I see the source view
    And the source is empty

  Scenario: The welcome page has a source
    When the app starts with no address given
    And I view the source
    Then I see the source view
