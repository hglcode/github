async function main() {
    const uri = process.argv.length > 2 ? process.argv[2] : null;
    const uri_test = 'https://raw.githubusercontent.com/hglcode/xshrc/refs/heads/main/README.md';

    try {
        const rsp = await fetch('https://api.akams.cn/github');
        const data = ((await rsp.json()) as { data?: { url: string; latency: number }[] })?.data?.sort(
            (a: { latency: number }, b: { latency: number }) => (a.latency < b.latency ? -1 : 1),
        );
        const proxies = (data || []).map((l: { url: string }) => l.url);
        const ps = proxies.map((l: string) =>
            fetch(`${l}/${uri_test}`)
                .then((r) => (uri ? `${l}/${uri}` : l))
                .catch((err) => {
                    console.error(`Error fetching URL ${l}:`, err);
                    return null;
                }),
        );
        const lnks = (await Promise.all(ps)).filter(Boolean);
        console.info([...lnks, uri].join(' '));
    } catch (error) {
        console.error('Error in main function:', error);
    }
}

main();
