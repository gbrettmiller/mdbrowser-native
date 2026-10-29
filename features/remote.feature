@remote
Feature: Browse remote Markdown
  As a reader of the web of Markdown
  I want to open Markdown pages by address and follow links between sites
  So that writing published anywhere reads like one web

  # Fixture: a std-only HTTP server on 127.0.0.1:0, fresh per scenario. It records each
  # request (path and headers). Routes:
  #   /index.md      text/markdown   "# Remote Home", "## Intro"
  #   /notes/a.md    text/markdown   "# Remote Note"        /notes/b.md   text/markdown "# Note B"
  #   /post          text/markdown   "# Post Without Extension"
  #   /plain.md      text/plain      "# Plain Served"       /plain.txt    text/plain    "# Not this"
  #   /untyped.md    (no type)       "# Untyped"            /page.html    text/html     "<h1>HTML</h1>"
  #   /fake.md       text/html       "<h1>Not Markdown</h1>"
  #   /moved.md      301 → /index.md                        /loop.md      302 → /loop.md
  #   /to-file.md    302 → file:///etc/hosts                /no-location.md 302, no Location
  #   /missing.md    404   /gone.md 410   /broken.md 500
  #   /latin1.md     text/markdown; charset=iso-8859-1      /bad-utf8.md  text/markdown, invalid bytes
  #   /huge.md       body over the world's size limit (the limit is lowered for the test)
  #   /slow.md       "# Slow"; held by the world until released
  # Steps assert on session state, recorded effects and the requests the server saw.

  Scenario: Open a remote page by address
    When I enter the URL of "/index.md"
    Then I see the document titled "Remote Home"
    And the address shows the URL of "/index.md"
    And the site was told this reader prefers Markdown

  Scenario Outline: Which responses are Markdown
    When I enter the URL of "<path>"
    Then I see the document titled "<title>"
    Examples:
      | path        | title                    |
      | /post       | Post Without Extension   |
      | /plain.md   | Plain Served             |
      | /untyped.md | Untyped                  |

  Scenario Outline: Responses that aren't Markdown
    When I enter the URL of "<path>"
    Then I see the error "notMarkdown"
    And the error shows the URL of "<path>"
    Examples:
      | path       |
      | /page.html |
      | /plain.txt |
      | /fake.md   |

  Scenario: Redirects are followed and the final address is shown
    When I enter the URL of "/moved.md"
    Then I see the document titled "Remote Home"
    And the address shows the URL of "/index.md"
    And History lists the URL of "/index.md" titled "Remote Home"

  Scenario Outline: Redirects that can't be followed
    When I enter the URL of "<path>"
    Then I see the error "badRedirect"
    Examples:
      | path            |
      | /loop.md        |
      | /to-file.md     |
      | /no-location.md |

  Scenario Outline: Fetch failures
    When I enter the URL of "<path>"
    Then I see the error "<code>"
    And the error shows the URL of "<path>"
    And the error says what to try next
    Examples:
      | path         | code        |
      | /missing.md  | notFound    |
      | /gone.md     | notFound    |
      | /broken.md   | serverError |
      | /latin1.md   | unreadable  |
      | /bad-utf8.md | unreadable  |
      | /huge.md     | unreadable  |

  Scenario: A site that refuses the connection
    When I enter a URL whose connection is refused
    Then I see the error "unreachable"

  Scenario: Reload after a failure tries again
    Given I entered the URL of "/broken.md" and see the error "serverError"
    And "/broken.md" now serves "# Fixed"
    When I reload
    Then I see the document titled "Fixed"

  Scenario: Reloading a remote page fetches it again
    Given I am viewing the URL of "/index.md"
    When I reload
    Then I see the document titled "Remote Home"
    And the site has seen 2 requests for "/index.md"

  Scenario: Reloading a redirected page goes straight to where it ended
    Given I am viewing the URL of "/moved.md"
    When I reload
    Then I see the document titled "Remote Home"
    And the site has seen 1 request for "/moved.md"
    And the site has seen 2 requests for "/index.md"

  Scenario: Back and forward between pages
    Given I viewed the URL of "/index.md" then the URL of "/notes/a.md"
    When I go back
    Then I see the document titled "Remote Home"
    And the address shows the URL of "/index.md"
    When I go forward
    Then I see the document titled "Remote Note"
    And the address shows the URL of "/notes/a.md"

  Scenario: Reopen a remote visit from History
    Given I viewed the URL of "/index.md" then the URL of "/notes/a.md"
    When I open the URL of "/index.md" from History
    Then I see the document titled "Remote Home"
    And the address shows the URL of "/index.md"

  Scenario: A slow fetch doesn't replace a newer page
    Given loading the URL of "/slow.md" is slow
    When I enter the URL of "/slow.md"
    And I enter the URL of "/index.md"
    And "/slow.md" finishes loading
    Then I see the document titled "Remote Home"
    And the address shows the URL of "/index.md"
    And the site has seen 1 request for "/slow.md"
