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
