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

  Scenario Outline: Going elsewhere leaves the source view
    Given I am viewing "index.md"
    And I viewed the source
    When <action>
    Then I see the rendered page

    Examples:
      | action                                  |
      | I follow "notes/a.md"                   |
      | I press Home                            |
      | I enter the absolute path of "notes/a.md" |

  Scenario: Back leaves the source view
    Given I viewed "index.md" then "notes/a.md"
    And I viewed the source
    When I go back
    Then I see the rendered page
    And I am still viewing "index.md"

  Scenario: Reload keeps the source view and shows the new text
    Given a file "live.md" containing "# Old\n"
    And I am viewing "live.md"
    And I viewed the source
    When the file "live.md" changes to "# Changed\n"
    And I reload
    Then I see the source view
    And the source is "# Changed\n"

  Scenario: A failed reload leaves the source view
    Given a file "gone.md" containing "# Gone\n"
    And I am viewing "gone.md"
    And I viewed the source
    When "gone.md" is deleted
    And I reload
    Then I see the error "notFound"
    And I am not in the source view

  Scenario: A page found by fallback shows the Markdown that was found
    Given I am viewing the URL of "/article"
    When I view the source
    Then the source is "# Article In Markdown\n"
    And the address shows the URL of "/article.md"

  Scenario: Reload after a fallback find stays in the source view
    Given I am viewing the URL of "/article"
    And I viewed the source
    When I reload
    Then I see the source view
    And the source is "# Article In Markdown\n"

  Scenario: Windows line endings and a byte order mark are not shown
    Given a file "crlf.md" with Windows line endings and a byte order mark
    And I am viewing "crlf.md"
    When I view the source
    Then the source is "# Title\n\nline one\nline two\n"

  Scenario: The source view is not a History entry and changes no settings
    Given I am viewing "index.md"
    When I view the source
    Then back is disabled
    And History holds 1 entries
    And the saved settings are the session's
