use name_resolver::get_name_url;

#[tokio::test]
async fn test_resolve() {
    const INPUT_OUTPUT: [(&str, &str); 4] = [
        (
            "boston",
            "https://ipfs.infura.io/ipfs/QmZk9uh2mqmXJFKu2Hq7kFRh93pA8GDpSZ6ReNqubfRKKQ",
        ),
        (
            "ARWV.boston",
            "https://arweave.net/KuB5jmew87_M2flH9f6ZpB9jlDv8hZSHPrmGUY8KqEk",
        ),
        (
            "sub.boston",
            "https://ipfs.infura.io/ipfs/QmeHUsLEdoEzTVuRxHcYxx6mXDqs9RhEawCS3a3AQTFFeM",
        ),
        (
            "ARWV.sub.boston",
            "https://arweave.net/VE2zcstYZ9ptHWQcQBrb4gOe6j162c7NdO8xy4OcWiE",
        ),
    ];
    // let name = "sub.boston";
    // let res = get_name_url(name).await.unwrap();
    // println!("{:?}", res.as_str());

    for (input, output) in INPUT_OUTPUT {
        let res = get_name_url(input).await.unwrap();
        assert_eq!(res.as_str(), output);
    }
}
