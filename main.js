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
    'https://cdn.akaere.online',
    'https://fastgit.cc',
    'https://free.cn.eu.org',
    'https://g.blfrp.cn',
    'https://gh.927223.xyz',
    'https://gh.b52m.cn',
    'https://gh.bugdey.us.kg',
    'https://gh.catmak.name',
    'https://gh.chjina.com',
    'https://gh.ddlc.top',
    'https://gh.dpik.top',
    'https://gh.felicity.ac.cn',
    'https://gh.fhjhy.top',
    'https://gh.idayer.com',
    'https://gh.inkchills.cn',
    'https://gh.jasonzeng.dev',
    'https://gh.llkk.cc',
    'https://gh.monlor.com',
    'https://gh.noki.icu',
    'https://gh.nxnow.top',
    'https://gh.sixyin.com',
    'https://gh.xxooo.cf',
    'https://gh.zwnes.xyz',
    'https://ghfast.top',
    'https://ghfile.geekertao.top',
    'https://ghm.078465.xyz',
    'https://ghp.keleyaa.com',
    'https://ghproxy.cxkpro.top',
    'https://ghproxy.imciel.com',
    'https://ghproxy.monkeyray.net',
    'https://ghproxy.net',
    'https://ghps.cc',
    'https://ghpxy.hwinzniej.top',
    'https://git.669966.xyz',
    'https://git.yylx.win',
    'https://github.chenc.dev',
    'https://github.dpik.top',
    'https://github.ednovas.xyz',
    'https://github.geekery.cn',
    'https://github.tbedu.top',
    'https://github.tmby.shop',
    'https://github.xxlab.tech',
    'https://gitproxy.127731.xyz',
    'https://gitproxy.click',
    'https://gitproxy.mrhjx.cn',
    'https://gp.zkitefly.eu.org',
    'https://j.1lin.dpdns.org',
    'https://j.1win.ggff.net',
    'https://jiashu.1win.eu.org',
    'https://proxy.yaoyaoling.net',
    'https://tvv.tw',
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
