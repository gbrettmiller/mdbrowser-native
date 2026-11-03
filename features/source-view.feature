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

  Scenario: Ctrl+U toggles the source view
    Given I am viewing "index.md"
    When I press "Ctrl+U"
    Then I see the source view
    When I press "Ctrl+U"
    Then I see the rendered page

  Scenario: Esc leaves the source view when no panel is open
    Given I am viewing "index.md"
    And I viewed the source
    When I press "Escape"
    Then I see the rendered page

  Scenario: Esc closes a panel before it leaves the source view
    Given I am viewing "index.md"
    And I viewed the source
    And the Settings panel is open
    When I press "Escape"
    Then the Settings panel is closed
    And I see the source view
    When I press "Escape"
    Then I see the rendered page

  Scenario: Esc on a page changes nothing
    Given I am viewing "index.md"
    When I press "Escape"
    Then I see the rendered page

  Scenario: Ctrl+U with no page shown does nothing
    Given I entered the absolute path of "missing.md" and see the error "notFound"
    When I press "Ctrl+U"
    Then I am not in the source view

  Scenario: The button names the other view
    Given I am viewing "index.md"
    Then the source button reads "Source"
    And the source button is enabled
    When I view the source
    Then the source button reads "Page"

  Scenario: The button is disabled when there is no page
    Given I entered the absolute path of "missing.md" and see the error "notFound"
    Then the source button is disabled

  Scenario: The source is set in the monospace font on the code background
    Given I am viewing "index.md"
    When I view the source
    Then the source is set in the monospace font on the code background

  Scenario: The reader's text size applies to the source
    Given I am viewing "index.md"
    And I viewed the source
    When I make the text larger 2 times
    Then the source is set at a larger size

  Scenario Outline: Scroll keys scroll the source view
    Given a document "long.md" that is 120 paragraphs long
    And I am viewing "long.md"
    And I viewed the source
    And the window is 600 pixels tall
    When I press "<key>"
    Then the page is <by>
    Examples:
      | key      | by                     |
      | PageDown | scrolled one page down |
      | Down     | scrolled one line down |
      | End      | at the end             |

  Scenario: The page keeps its place while the source is shown
    Given a document "long.md" that is 120 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    And I pressed "PageDown"
    When I view the source
    Then the page is at the top
    When I press "End"
    And I view the page
    Then the page is scrolled one page down

  Scenario: The source keeps its place while the page is shown
    Given a document "long.md" that is 120 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    And I viewed the source
    And I pressed "PageDown"
    When I view the page
    And I view the source
    Then the page is scrolled one page down

  Scenario: Reload shows the source from the top
    Given a document "long.md" that is 120 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    And I viewed the source
    And I pressed "PageDown"
    When I reload
    Then I see the source view
    And the page is at the top

  Scenario: Leaving a page forgets where its source was
    Given a document "long.md" that is 120 paragraphs long
    And I am viewing "long.md"
    And the window is 600 pixels tall
    And I viewed the source
    And I pressed "PageDown"
    When I view the page
    And I follow "index.md"
    Then the page is at the top
