@images
Feature: Images
  As a reader of the web of Markdown
  I want the pictures in a post drawn in the page
  So that posts read as their writers meant, with nothing else running

  # Scenarios land step by step with phase 2b (vector/plans/phase-2b-images.md).
  # Until images are drawn (step 8), every image shows its description.

  Scenario: An image in the middle of a paragraph keeps reading order
    Given a file "mid.md" containing "Before ![a box](none.png) after"
    When I view "mid.md"
    Then the document text is "Before\n[image: a box]\nafter"

  Scenario: An image in a heading stays its description
    Given a file "h.md" containing "# Title ![logo](none.png)"
    When I view "h.md"
    Then I see the document titled "Title [image: logo]"
    And no image is shown

  Scenario: Web images load automatically on first run
    Then web images are set to "automatic"

  Scenario Outline: The image setting is remembered
    Given I set web images to "<mode>"
    When I restart the app
    Then web images are set to "<mode>"
    Examples:
      | mode     |
      | on click |
      | off      |

  # ── step 8: drawn images (local files, then the web) ───────────────────────

  Scenario: A local image is drawn at its own size
    Given an image file "pic.png" of 12 by 8 pixels
    And a file "local.md" containing "![a red box](pic.png)"
    When I view "local.md"
    Then the image "a red box" is shown at 12 by 8

  Scenario: A web image on a web page is drawn
    Given the site serves "/post.md" containing "![red](img/red.png)"
    When I enter the URL of "/post.md"
    Then the image "red" is shown at 40 by 20
    And the site was asked for an image at "/img/red.png" with no cookie and no referrer

  Scenario Outline: Formats that are drawn
    Given the site serves "/post.md" containing "![x](<src>)"
    When I enter the URL of "/post.md"
    Then the image "x" is shown at <width> by <height>
    Examples:
      | src               | width | height |
      | /img/alpha.png    | 8     | 8      |
      | /img/anim.gif     | 30    | 30     |
      | /img/photo.jpg    | 24    | 24     |
      | /img/photo.webp   | 24    | 24     |
      | /img/untyped.png  | 10    | 10     |
      | /img/mislabel.png | 10    | 10     |
      | /img/moved.png    | 40    | 20     |

  Scenario Outline: Images that can't be shown say why
    Given the site serves "/post.md" containing "Before ![a logo](<src>) after"
    When I enter the URL of "/post.md"
    Then the image "a logo" is not shown and says "<note>"
    And the document text is "Before\n[image: a logo]\nafter"
    Examples:
      | src              | note                                           |
      | /img/logo.svg    | This image's format isn't supported.           |
      | /img/page.png    | This image's format isn't supported.           |
      | /img/missing.png | Couldn't load this image. Click or press Ctrl+Shift+I to try again. |
      | /img/tall.png    | This image is too large to show.               |
      | /img/huge.png    | This image is too large to show.               |
      | /img/to-file.png | This page can't show this image.               |

  Scenario: An image wider than the column is scaled down
    Given the site serves "/post.md" containing "![wide](img/wide.png)"
    When I enter the URL of "/post.md"
    Then the image "wide" is as wide as the column, with its aspect ratio kept

  Scenario: A local file can show web images, including from the local network
    Given a file "local.md" linking an image to the URL of "/img/red.png"
    When I view "local.md"
    Then the image is shown at 40 by 20

  Scenario Outline: A web page can't show files from the disk
    Given an image file "secret.png" of 4 by 4 pixels
    And the site serves "/post.md" containing an image from <source>
    When I enter the URL of "/post.md"
    Then the image is not shown and says "This page can't show this image."
    And nothing was read from disk
    Examples:
      | source                          |
      | the file:// URL of "secret.png" |
      | "C:\\secret.png"                |
      | "\\\\host\\share\\secret.png"   |
      | "data:image/png;base64,AAAA"    |

  Scenario: A repeated image is fetched once
    Given the site serves "/post.md" containing "![a](img/red.png) ![b](img/red.png)"
    When I enter the URL of "/post.md"
    Then the image "a" is shown at 40 by 20
    And the image "b" is shown at 40 by 20
    And the site has seen 1 request for "/img/red.png"

  Scenario: A failed image can be tried again
    Given the site serves "/post.md" containing "![x](img/flaky.png)"
    And I am viewing the URL of "/post.md"
    And the image "x" is not shown and says "Couldn't load this image. Click or press Ctrl+Shift+I to try again."
    When I click the note under "x"
    Then the image "x" is shown at 40 by 20

  Scenario: Reloading retries images that failed
    Given the site serves "/post.md" containing "![x](img/flaky.png)"
    And I am viewing the URL of "/post.md"
    When I reload
    Then the image "x" is shown at 40 by 20

  Scenario: Copying across an image copies its description
    Given an image file "pic.png" of 12 by 8 pixels
    And a file "c.md" containing "Before\n\n![a red box](pic.png)\n\nafter"
    And I am viewing "c.md"
    When I select everything and copy
    Then the copied text is "Before\n[image: a red box]\nafter"

  Scenario: HTML img is never an image
    Given the site serves "/post.md" containing "<img src='img/red.png'> text"
    When I enter the URL of "/post.md"
    Then no image is shown
    And the site has seen 0 requests for "/img/red.png"

  # ── step 9: the setting, clicks, links, loading and the cache ──────────────

  Scenario: On click: web images wait, local images load
    Given I set web images to "on click"
    And an image file "pic.png" of 12 by 8 pixels
    And a file "both.md" linking an image to "pic.png" and an image to the URL of "/img/red.png"
    When I view "both.md"
    Then the image "pic.png" is shown at 12 by 8
    And the image "red.png" is not shown and says "Click to load, or press Ctrl+Shift+I for this page's."
    And the site has seen 0 requests for "/img/red.png"
    When I click the note under "red.png"
    Then the image "red.png" is shown at 40 by 20
    And the site has seen 1 request for "/img/red.png"

  Scenario: Off: web images are not requested and can't be clicked
    Given I set web images to "off"
    And the site serves "/post.md" containing "![red](img/red.png)"
    When I enter the URL of "/post.md"
    Then the image "red" is not shown and says "Web images are off. Press Ctrl+Shift+I to load this page's."
    And clicking the note under "red" does nothing
    And the site has seen 0 requests for "/img/red.png"

  Scenario Outline: Switching to automatic loads the images on the page
    Given I set web images to "<mode>"
    And the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    And the site has seen 0 requests for "/img/red.png"
    When I set web images to "automatic"
    Then the image "red" is shown at 40 by 20
    Examples:
      | mode     |
      | on click |
      | off      |

  Scenario: Switching off keeps images already shown
    Given the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    When I set web images to "off"
    Then the image "red" is shown at 40 by 20

  Scenario: Switching from off to on click makes images clickable
    Given I set web images to "off"
    And the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    When I set web images to "on click"
    Then the image "red" is not shown and says "Click to load, or press Ctrl+Shift+I for this page's."

  Scenario: Loading this page's images leaves the setting alone
    Given I set web images to "off"
    And the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    When I load this page's images
    Then the image "red" is shown at 40 by 20
    And web images are set to "off"

  Scenario: A linked image follows its link
    Given the site serves "/post.md" containing "[![red](img/red.png)](https://example.com/)"
    And I am viewing the URL of "/post.md"
    When I click the image "red"
    Then the system opened "https://example.com/"

  Scenario: A linked image waiting for a click
    Given I set web images to "on click"
    And the site serves "/post.md" containing "[![red](img/red.png)](https://example.com/)"
    And I am viewing the URL of "/post.md"
    When I click the note under "red"
    Then the image "red" is shown at 40 by 20
    And nothing is opened externally
    When I click the description "red"
    Then the system opened "https://example.com/"

  Scenario: At most four images load at once
    Given the site serves "/gallery.md" with images "/img/slow-1.png" to "/img/slow-6.png"
    And loading each image under "/img/slow-" is slow
    When I enter the URL of "/gallery.md"
    Then the site has seen 4 image requests
    When "/img/slow-1.png" finishes loading
    Then the site has seen 5 image requests

  Scenario: Leaving a page drops its waiting loads
    Given the site serves "/gallery.md" with images "/img/slow-1.png" to "/img/slow-6.png"
    And loading each image under "/img/slow-" is slow
    And I entered the URL of "/gallery.md"
    When I enter the URL of "/index.md"
    And every slow image finishes loading
    Then the site has seen 4 image requests
    And I see the document titled "Remote Home"
    And no image is shown

  Scenario: Going back shows cached images without fetching again
    Given the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    And the image "red" is shown at 40 by 20
    And I enter the URL of "/index.md"
    When I go back
    Then the image "red" is shown at 40 by 20
    And the site has seen 1 request for "/img/red.png"

  Scenario: An image dropped from the cache is fetched again
    Given the image cache has room for one 40 by 20 image
    And the site serves "/a.md" containing "![red](img/red.png)"
    And the site serves "/b.md" containing "![photo](img/photo.jpg)"
    And I viewed the URL of "/a.md" then the URL of "/b.md"
    When I go back
    Then the image "red" is shown at 40 by 20
    And the site has seen 2 requests for "/img/red.png"
