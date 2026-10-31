@frontmatter
Feature: Shared front matter
  As a reader of the web of Markdown
  I want to see who wrote a post and when, and move to the posts around it
  So that a blog reads as a blog, in the presentation I chose

  # ADR-006. The fixture is the home of browse.feature (index.md, notes/a.md), plus the
  # files each scenario writes, and for the remote scenarios the routes of remote.feature plus
  #   /blog/two.md  text/markdown  urlPrev "one.md", urlNext "../three.md"
  #   /blog/one.md  text/markdown  "# Post One"       /three.md  text/markdown "# Post Three"
  #
  # Steps check what the session reports (the document's front matter, the offered
  # navigation, and the effects sent to the opener). Where the byline, description and
  # previous/next row are drawn, and in what type, is the layout's job: golden snapshots and
  # scripts/shoot.sh check that, not these steps.
  #
  # In a file, \n inside a quoted string is a newline.

  Scenario: Author and date make a byline
    Given a file "post.md" containing "---\nauthor: Dave Winer\npubDate: Sun, 31 Mar 2024 16:17:30 GMT\n---\nText"
    When I view "post.md"
    Then the byline reads "Dave Winer · 31 March 2024"

  Scenario: The site's name joins the byline
    Given a file "post.md" containing "---\nauthor: Dave Winer\npubDate: Sun, 31 Mar 2024 16:17:30 GMT\nsource:\n  url: http://scripting.com/rss.xml\n  title: Scripting News\n---\nText"
    When I view "post.md"
    Then the byline reads "Dave Winer · 31 March 2024 · Scripting News"

  Scenario: Fields that are missing drop out of the byline
    Given a file "post.md" containing "---\nauthor: Dave Winer\n---\nText"
    When I view "post.md"
    Then the byline reads "Dave Winer"

  Scenario: With no author, date or site there is no byline
    Given a file "post.md" containing "---\ntitle: Plain\n---\nText"
    When I view "post.md"
    Then there is no byline

  Scenario: A date that isn't RFC 822 is shown as written
    Given a file "post.md" containing "---\nauthor: Dave Winer\npubDate: last Tuesday-ish\n---\nText"
    When I view "post.md"
    Then the byline reads "Dave Winer · last Tuesday-ish"

  Scenario: The description leads the post
    Given a file "post.md" containing "---\ndescription: A short summary\n---\nText"
    When I view "post.md"
    Then the description reads "A short summary"

  Scenario: A description the post already opens with is shown once
    Given a file "post.md" containing "---\ndescription: Text\n---\nText"
    When I view "post.md"
    Then the text "Text" appears once on the page

  Scenario: Previous and next move through a blog
    Given a file "blog/two.md" containing "---\nurlPrev: one.md\nurlNext: ../three.md\n---\n# Post Two"
    And a file "blog/one.md" containing "# Post One"
    And a file "three.md" containing "# Post Three"
    When I view "blog/two.md"
    Then the page offers Previous and Next
    When I choose Next
    Then I see the document titled "Post Three"
    And I am still viewing "three.md"
    When I go back
    And I choose Previous
    Then I see the document titled "Post One"

  Scenario: Next is recorded in History like any other visit
    Given a file "blog/two.md" containing "---\nurlNext: three.md\n---\n# Post Two"
    And a file "blog/three.md" containing "# Post Three"
    And I am viewing "blog/two.md"
    When I choose Next
    Then History lists "blog/three.md" titled "Post Three"

  Scenario: The newest post offers only Previous
    Given a file "post.md" containing "---\nurlPrev: older.md\n---\nText"
    When I view "post.md"
    Then the page offers Previous only

  Scenario: A post with neither offers no navigation
    Given a file "post.md" containing "---\nauthor: Dave Winer\n---\nText"
    When I view "post.md"
    Then the page offers no Previous or Next

  Scenario: A previous post that isn't there shows the usual error
    Given a file "post.md" containing "---\nurlPrev: gone.md\n---\nText"
    And I am viewing "post.md"
    When I choose Previous
    Then I see the error "notFound"

  Scenario: Previous and next work over http
    Given I am viewing the URL of "/blog/two.md"
    When I choose Next
    Then I see the document titled "Post Three"
    And the address shows the URL of "/three.md"

  Scenario: An enclosure is shown as a line naming the file
    Given a file "ep.md" containing "---\nenclosure: http://example.com/show.m4a\nenclosureType: audio/mpeg\nenclosureLength: 6374173\n---\nA show"
    And I am viewing "ep.md"
    Then the enclosure line reads "Listen: show.m4a (audio/mpeg, 6.1 MB)"

  Scenario: An enclosure without a type or length still shows
    Given a file "ep.md" containing "---\nenclosure: http://example.com/show.m4a\n---\nA show"
    When I view "ep.md"
    Then the enclosure line reads "Listen: show.m4a"

  Scenario Outline: Only web addresses are handed to the system
    Given a file "ep.md" containing "---\nenclosure: <enclosure>\n---\nA show"
    When I view "ep.md"
    Then there is no enclosure line
    Examples:
      | enclosure              |
      | file:///etc/passwd     |
      | javascript:alert(1)    |
      | not a url              |

  Scenario Outline: A value of the wrong kind is dropped, and the rest still shows
    Given a file "post.md" containing "---\nauthor: <bad>\npubDate: Sun, 31 Mar 2024 16:17:30 GMT\nurlNext: <bad>\n---\n# Still Here"
    When I view "post.md"
    Then I see the document titled "Still Here"
    And the byline reads "31 March 2024"
    And the page offers no Previous or Next
    Examples:
      | bad        |
      | [a, b]     |
      | {x: 1}     |
      | ''         |

  Scenario: Keys we don't know are ignored
    Given a file "post.md" containing "---\ngenerator: opmlToMarkdown v0.5.11\nurlvideo: http://x/v.mp4\nurltoot: http://x/t\n---\n# Quiet"
    When I view "post.md"
    Then I see the document titled "Quiet"
    And there is no byline
    And the page offers no Previous or Next

  Scenario: Malformed front matter doesn't stop the page
    Given a file "post.md" containing "---\nauthor: [unclosed\n---\n# Body Survives"
    When I view "post.md"
    Then I see the document titled "Body Survives"
    And there is no byline

  Scenario: Image fields add no image beyond the body's
    Given a file "post.md" containing "---\nimage: http://example.com/a.png\ninlineImage: http://example.com/b.png\nmetaImage: http://example.com/c.png\n---\nText"
    When I view "post.md"
    Then the page shows no images
