@remote
Feature: Browse remote Markdown
  As a reader of the web of Markdown
  I want to open Markdown pages by address and follow links between sites
  So that writing published anywhere reads like one web

  # Fixture: a std-only HTTP server on 127.0.0.1:0, fresh per scenario. It records each
  # request (path and headers). Routes:
  #   /index.md      text/markdown   "# Remote Home", "## Intro"; links notes/a.md, /notes/b.md, #intro,
  #                                  /missing.md, https://example.com/about, mailto:a@b.co
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

  Scenario Outline: A page that isn't Markdown can go to the system browser
    When I enter the URL of "<path>"
    Then I see the error "notMarkdown"
    And the error shows the URL of "<path>"
    When I choose to open it in the browser
    Then the system opened the URL of "<path>"
    Examples:
      | path       |
      | /page.html |
      | /plain.txt |

  Scenario: A followed link to a page that isn't Markdown offers the system browser
    Given I am viewing the URL of "/index.md"
    When I follow "/fake.md"
    Then I see the error "notMarkdown"
    When I choose to open it in the browser
    Then the system opened the URL of "/fake.md"

  Scenario: A local file that isn't Markdown offers no browser
    When I enter the absolute path of "data.txt"
    Then I see the error "notMarkdown"
    And I am not offered to open it in the browser

  Scenario: Other failures offer no browser
    When I enter the URL of "/missing.md"
    Then I see the error "notFound"
    And I am not offered to open it in the browser

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

  Scenario Outline: Links in a remote page resolve against its address
    Given I am viewing the URL of "/notes/a.md"
    When I follow "<href>"
    Then I see the document titled "<title>"
    And the address shows the URL of "<path>"
    Examples:
      | href         | title       | path        |
      | b.md         | Note B      | /notes/b.md |
      | ../index.md  | Remote Home | /index.md   |
      | /notes/b.md  | Note B      | /notes/b.md |

  Scenario: A fragment link in a remote page scrolls
    Given I am viewing the URL of "/index.md"
    When I follow "#intro"
    Then the pending fragment is "intro"
    And the address shows the URL of "/index.md"

  Scenario: A link to a missing page shows an error, and back returns
    Given I am viewing the URL of "/index.md"
    When I follow "/missing.md"
    Then I see the error "notFound"
    When I go back
    Then I see the document titled "Remote Home"

  Scenario Outline: Non-Markdown links leave the app
    Given I am viewing the URL of "/index.md"
    When I follow "<href>"
    Then the system opened "<href>"
    And I am still viewing the URL of "/index.md"
    Examples:
      | href                      |
      | https://example.com/about |
      | mailto:a@b.co             |

  Scenario Outline: A remote page can't open local files
    Given a file "secret.md" containing "# Secret"
    And I am viewing the URL of "/index.md"
    When I follow <link>
    Then I am told "Web pages can't open files on your computer."
    And I am still viewing the URL of "/index.md"
    And nothing was read from disk
    Examples:
      | link                                 |
      | the file:// URL of "secret.md"       |
      | "C:\\secret.md"                     |
      | "\\\\host\\share\\secret.md"         |

  Scenario: An absolute path in a remote page is a path on the same site
    Given I am viewing the URL of "/index.md"
    When I follow "/etc/hosts.md"
    Then I see the error "notFound"
    And the site has seen 1 request for "/etc/hosts.md"
    And nothing was read from disk

  Scenario: A local file links to a remote page
    Given a file "links.md" linking to the URL of "/index.md"
    And I am viewing "links.md"
    When I follow the URL of "/index.md"
    Then I see the document titled "Remote Home"

  Scenario: A local file's non-Markdown web link goes to the system browser
    Given a file "web.md" linking to "https://example.com/"
    And I am viewing "web.md"
    When I follow "https://example.com/"
    Then the system opened "https://example.com/"

  Scenario: Back and forward between files and pages
    Given a file "links.md" linking to the URL of "/index.md"
    And I viewed "links.md" then the URL of "/index.md"
    When I go back
    Then the address shows the absolute path of "links.md"
    When I go forward
    Then I see the document titled "Remote Home"
