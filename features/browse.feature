@browse
Feature: Browse local Markdown files
  As a reader of Markdown
  I want to open local .md files by path and follow links between them
  So that my notes feel like a small web

  # Ported from ~/projects/mdbrowser (2026-10-28). Fixture: a fresh temporary home
  # per scenario containing index.md, notes/a.md and data.txt. Steps assert on
  # session state and recorded effects (reads, opener calls), never on pixels.

  Scenario Outline: Open a file by address
    When I enter <form> of "index.md"
    Then I see the document titled "Home"
    And the address shows the absolute path of "index.md"
    Examples:
      | form                      |
      | the absolute path         |
      | the home-relative ~/ path |
      | the file:// URL           |

  Scenario Outline: Unusable addresses
    When I enter "<input>"
    Then I see the error "<code>"
    And the error shows the address "<input>"
    And nothing was read from disk
    Examples:
      | input                         | code              |
      |                               | empty             |
      | notes/a.md                    | notAbsolute       |
      | https://example.com/readme.md | unsupportedScheme |

  Scenario: Not a Markdown file is rejected before reading
    When I enter the absolute path of "data.txt"
    Then I see the error "notMarkdown"
    And nothing was read from disk

  Scenario: Missing file
    When I enter the absolute path of "missing.md"
    Then I see the error "notFound"

  Scenario: Unreadable file
    Given "secret.md" exists with invalid UTF-8 content
    When I enter the absolute path of "secret.md"
    Then I see the error "unreadable"

  Scenario Outline: Title fallbacks
    Given a file "t.md" containing "<content>"
    When I enter the absolute path of "t.md"
    Then I see the document titled "<title>"
    Examples:
      | content                           | title   |
      | # Heading                         | Heading |
      | plain text                        | t.md    |
      | ---\ntitle: [bad\n---\n# Fallback | Fallback |

  Scenario Outline: Raw HTML is stripped, its text kept
    Given a file "h.md" containing "<content>"
    When I enter the absolute path of "h.md"
    Then the document text is "<text>"
    Examples:
      | content                          | text              |
      | before <b>bold</b> after         | before bold after |
      | <img src=x onerror=alert(1)>tail | tail              |
      | <!-- a comment -->kept           | kept              |
      | <a title='a>b'>x</a>             | x                 |
      | <script>alert(1)</script>        | alert(1)          |
      | `<b>code</b>` stays              | <b>code</b> stays |
      | a < b and <3                     | a < b and <3      |

  Scenario: HTML in a heading is stripped from the title
    Given a file "t.md" containing "# <b>Title</b>"
    When I enter the absolute path of "t.md"
    Then I see the document titled "Title"

  Scenario: Opener failure shows a notice
    Given the system opener fails
    And I am viewing "index.md"
    When I follow "https://example.com"
    Then I am told "Couldn't open this link."
    And I am still viewing "index.md"

  Scenario: Follow a relative Markdown link
    Given I am viewing "index.md"
    When I follow "notes/a.md"
    Then I see the document titled "Note A"
    And the address shows the absolute path of "notes/a.md"

  Scenario: Follow a link with a fragment into another document
    Given I am viewing "notes/a.md"
    When I follow "../index.md#intro"
    Then I see the document titled "Home"
    And the pending fragment is "intro"

  Scenario: Fragment-only link stays in the document
    Given I am viewing "index.md"
    When I follow "#intro"
    Then I am still viewing "index.md"
    And the pending fragment is "intro"
    And back is disabled

  Scenario: Broken relative link
    Given I am viewing "index.md"
    When I follow "gone.md"
    Then I see the error "notFound"
    And the address shows the absolute path of "gone.md"

  Scenario: Non-Markdown local link does nothing
    Given I am viewing "index.md"
    When I follow "data.txt"
    Then I am still viewing "index.md"
    And nothing is opened externally
    And I am told "Can't open this kind of link yet"

  Scenario Outline: External links leave the app
    Given I am viewing "index.md"
    When I follow "<href>"
    Then "<href>" is opened externally
    And I am still viewing "index.md"
    Examples:
      | href                |
      | https://example.com |
      | mailto:a@b.co       |

  Scenario Outline: Other schemes never reach the opener
    Given I am viewing "index.md"
    When I follow "<href>"
    Then nothing is opened externally
    And I am still viewing "index.md"
    Examples:
      | href                |
      | ftp://x.org/f       |
      | javascript:alert(1) |
      | file:///etc/passwd  |
      | tel:123             |
      | data:text/html,hi   |

  Scenario: Fresh session has nowhere to go
    Given I am viewing "index.md"
    Then back is disabled
    And forward is disabled
    When I go back
    Then I am still viewing "index.md"

  Scenario: Going back
    Given I viewed "index.md" then "notes/a.md"
    When I go back
    Then I see the document titled "Home"
    And forward is enabled

  Scenario: Going forward
    Given I viewed "index.md" then "notes/a.md" and went back
    When I go forward
    Then I see the document titled "Note A"
    And forward is disabled

  Scenario: Navigating after going back drops forward entries
    Given I viewed "index.md" then "notes/a.md" and went back
    When I enter the absolute path of "notes/a.md"
    Then forward is disabled

  Scenario: Reload picks up changes and keeps the stack
    Given I am viewing "index.md"
    When "index.md" is changed to have title "Home v2"
    And I reload
    Then I see the document titled "Home v2"
    And back is disabled

  Scenario: Reload of a deleted file
    Given I viewed "index.md" then "notes/a.md"
    When "notes/a.md" is deleted
    And I reload
    Then I see the error "notFound"
    When I go back
    Then I see the document titled "Home"

  Scenario: Errors are in Back/Forward but not History
    Given I viewed "index.md"
    When I enter the absolute path of "missing.md"
    And I go back
    Then I see the document titled "Home"
    And History does not list "missing.md"

  Scenario: History survives restart
    Given I viewed "index.md" then "notes/a.md"
    When I restart the app
    Then History lists "Note A" then "Home"

  Scenario: Revisiting moves an entry to the top without duplicating it
    Given I viewed "index.md" then "notes/a.md" then "index.md"
    Then History lists "Home" then "Note A"

  Scenario: History is capped
    Given History already holds 500 distinct documents
    When I view "index.md"
    Then History holds 500 entries
    And the oldest document is not listed

  Scenario: Revisiting at the cap evicts nothing
    Given History already holds 500 distinct documents including "index.md"
    When I view "index.md"
    Then History holds 500 entries
    And "Home" is first in History

  Scenario: Stale History entry
    Given History lists "notes/a.md"
    And "notes/a.md" is deleted
    When I open the History entry for "notes/a.md"
    Then I see the error "notFound"

  Scenario: Clear History
    Given I viewed "index.md"
    When I clear History
    Then History is empty

  Scenario: Superseded load is discarded
    Given loading "index.md" is slow
    When I enter the absolute path of "index.md"
    And I enter the absolute path of "notes/a.md" before it finishes
    Then I see the document titled "Note A"
    And "index.md" does not replace it when it finishes

  Scenario: Reload of an error page retries
    Given I entered the absolute path of "later.md" and see the error "notFound"
    When "later.md" is created
    And I reload
    Then I see the document titled "later.md"

