# Philologus Fulltext iOS

This is a Rust library with Swift bindings to add Tantivy fulltext search functionality to Philologus iOS app.

./build-iOS.sh to build the iOS version of the library.

add the swift file to the project.
add the framework folder to the project.

add the tantivy index to the project.
  tantivy index folder must be added to xcode as a Folder, not as a Group.
  must be included in the app target
  Build rules under properties for folder: "Apply Once to Folder"
  must copy folder to documents directory because of permissions error when accessing in main bundle

let tantivy_index = "tantivy-data"
copyBundleFolderToDocuments(folderName: tantivy_index)

let query = "example"
let page = 1

let fileManager = FileManager.default
let documentsDirectory = fileManager.urls(for: .documentDirectory, in: .userDomainMask).first!
let folderURL = documentsDirectory.appendingPathComponent(tantivy_index)

//let folderURL = Bundle.main.doc resourceURL?.appendingPathComponent("tantivy-data")
let folderPath = folderURL.path
let index_path = folderPath
let r = realFullTextQuery(query: query, page: UInt32(page), indexPath: index_path)
print(r)

func copyBundleFolderToDocuments(folderName: String) {
    let fileManager = FileManager.default

    // Get the URL for the source folder in the app bundle
    guard let sourceURL = Bundle.main.url(forResource: folderName, withExtension: nil) else {
        print("Source folder not found in bundle.")
        return
    }

    // Get the URL for the destination in the Documents directory
    guard let documentsURL = fileManager.urls(for: .documentDirectory, in: .userDomainMask).first else {
        print("Documents directory not found.")
        return
    }
    let destinationURL = documentsURL.appendingPathComponent(folderName, isDirectory: true)

    // Check if the folder already exists in the Documents directory to prevent overwrites
    if fileManager.fileExists(atPath: destinationURL.path) {
        print("Folder already exists in Documents directory. Skipping copy.")
        return
    }

    do {
        // Copy the entire folder from the bundle to the Documents directory
        try fileManager.copyItem(at: sourceURL, to: destinationURL)
        print("Successfully copied folder to Documents directory.")
    } catch {
        print("Error copying folder: \(error.localizedDescription)")
    }
}
