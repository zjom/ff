import XCTest
import SwiftTreeSitter
import TreeSitterFf

final class TreeSitterFfTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_ff())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading ff grammar")
    }
}
