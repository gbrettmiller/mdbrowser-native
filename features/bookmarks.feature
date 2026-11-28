@bookmarks
Feature: Bookmarks
  As a reader of Markdown
  I want to keep the pages I come back to, in the order I choose
  So that I can reach them without searching History

  # The charter's bookmarks menu (bonBon, "The shell"), drawn as a side panel like
  # History. A bookmark is an address and the page's title when it was added. The
  # list is the reader's own order: new ones go at the end, and each can move up or
  # down. Same fixture as browse.feature.

  Scenario: Bookmark the page you are reading
    Given I am viewing "index.md"
    When I bookmark this page
    Then this page is bookmarked
    And Bookmarks lists "Home"

  Scenario: Bookmarking again removes it
    Given I am viewing "index.md"
    And I bookmarked this page
    When I bookmark this page
    Then this page is not bookmarked
    And Bookmarks is empty

  Scenario: New bookmarks go at the end
    Given I am viewing "index.md"
    And I bookmarked this page
    When I view "notes/a.md"
    And I bookmark this page
    Then Bookmarks lists "Home" then "Note A"

  Scenario: Move a bookmark up and down
    Given I bookmarked "index.md" then "notes/a.md"
    When I move "Note A" up
    Then Bookmarks lists "Note A" then "Home"
    When I move "Note A" down
    Then Bookmarks lists "Home" then "Note A"

  Scenario: Moving past either end changes nothing
    Given I bookmarked "index.md" then "notes/a.md"
    When I move "Home" up
    And I move "Note A" down
    Then Bookmarks lists "Home" then "Note A"

  Scenario: Remove a bookmark from the list
    Given I bookmarked "index.md" then "notes/a.md"
    When I remove the bookmark "Home"
    Then Bookmarks lists only "Note A"

  Scenario: Open a bookmark
    Given I bookmarked "index.md" then "notes/a.md"
    When I open the bookmark "Home"
    Then I see the document titled "Home"

  Scenario: Bookmarks and their order survive restart
    Given I bookmarked "index.md" then "notes/a.md"
    And I moved "Note A" up
    When I restart the app
    Then Bookmarks lists "Note A" then "Home"

  Scenario: Visiting a bookmark leaves its place and title alone
    Given I bookmarked "index.md" then "notes/a.md"
    When I view "index.md"
    Then Bookmarks lists "Home" then "Note A"

  Scenario: Clearing History leaves Bookmarks
    Given I bookmarked "index.md" then "notes/a.md"
    When I clear History
    Then Bookmarks lists "Home" then "Note A"

  Scenario: Built-in pages and errors can't be bookmarked
    When I press Home
    And I bookmark this page
    Then Bookmarks is empty
    When I enter the absolute path of "missing.md"
    And I bookmark this page
    Then Bookmarks is empty

  Scenario: Bookmarks and History share a side
    Given the window is 1200 pixels wide
    And the History panel is open
    When I open the Bookmarks panel
    Then the History panel is closed
    And the Bookmarks panel is open
