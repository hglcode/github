"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
const proxies = [
    'https://gh.llkk.cc',
    'https://gh.zwnes.xyz',
    'https://ghps.cc',
    'https://github.dpik.top',
    'https://gh.chjina.com',
    'https://cccccccccccccccccccccccccccccccccccccccccccccccccccc.cc',
    'https://ghpxy.hwinzniej.top',
    'https://ghproxy.net',
    'https://ghproxy.fangkuai.fun',
    'https://free.cn.eu.org',
    'https://gh.fhjhy.top',
    'https://git.yylx.win',
    'https://github-proxy.teach-english.tech',
    'https://ghproxy.cc',
    'https://ghproxy.imciel.com',
    'https://j.1win.ggff.net',
    'https://github.geekery.cn',
    'https://ghfile.geekertao.top',
    'https://ghfast.top',
    'https://github.chenc.dev',
    'https://hub.gitmirror.com',
    'https://gh-proxy.com',
    'https://cf.ghproxy.cc',
    'https://fastgit.cc',
    'https://g.blfrp.cn',
    'https://github.tbedu.top',
    'https://gh.sixyin.com',
    'https://gh-proxy.top',
    'https://gh.bugdey.us.kg',
    'https://ghf.xn--eqrr82bzpe.top',
    'https://gh.idayer.com',
    'https://gh.5050net.cn',
    'https://gh.wsmdn.dpdns.org',
    'https://github.xxlab.tech',
    'https://git.669966.xyz',
    'https://gh.monlor.com',
    'https://gh.ddlc.top',
    'https://jiashu.1win.eu.org',
    'https://ghproxy.cn',
    'https://gitproxy.mrhjx.cn',
    'https://ghm.078465.xyz',
    'https://ghproxy.cxkpro.top',
    'https://gh.xxooo.cf',
    'https://cdn.akaere.online',
    'https://gh.dpik.top',
    'https://gh.927223.xyz',
    'https://gh.felicity.ac.cn',
    'https://ghp.keleyaa.com',
    'https://github.ednovas.xyz',
    'https://gh-proxy.net',
    'https://github-proxy.memory-echoes.cn',
    'https://gh.inkchills.cn',
    'https://proxy.yaoyaoling.net',
    'https://gitproxy.127731.xyz',
    'https://j.1lin.dpdns.org',
    'https://ghproxy.cfd',
    'https://tvv.tw',
    'https://gh.jasonzeng.dev',
    'https://gh.catmak.name',
    'https://ghproxy.monkeyray.net',
    'https://gp.zkitefly.eu.org',
    'https://gh.b52m.cn',
    'https://cdn.gh-proxy.com',
    'https://gh.noki.icu',
    'https://ghproxy.it',
    'https://ghproxy.it',
    'https://gh-proxy.com',
    'https://ghproxy.cfd',
    'https://ghproxy.it',
    'https://ghproxy.it',
];
const uri_test = 'https://raw.githubusercontent.com/hglcode/xshrc/refs/heads/main/README.md';
function main() {
    return __awaiter(this, void 0, void 0, function* () {
        const uri = (process.argv.length > 2) ? process.argv[2] : null;
        try {
            const ps = proxies.map((l) => fetch(`${l}/${uri_test}`)
                .then((r) => uri ? `${l}/${uri}` : l)
                .catch((err) => {
                console.error(`Error fetching URL ${l}:`, err);
                return null;
            }));
            const lnks = (yield Promise.all(ps)).filter(Boolean);
            console.debug([...lnks, uri].join(' '));
        }
        catch (error) {
            console.error('Error in main function:', error);
        }
    });
}
main();
//# sourceMappingURL=main.js.map
