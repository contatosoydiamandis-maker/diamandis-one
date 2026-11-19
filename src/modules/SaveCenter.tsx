import { HardDrive, Link2, ShieldCheck } from "lucide-react";

export default function SaveCenter() {
  return <section className="save-center-page">
    <div className="save-center-hero"><div className="save-center-icon"><HardDrive size={26}/></div><div><span className="tool-kicker">PROTEÇÃO DE PROGRESSO</span><h3>Seu Save Manager será o núcleo deste módulo.</h3><p>O Save Center está sendo preparado para receber a implementação existente, sem criar um segundo mecanismo de backup concorrente.</p></div></div>
    <article className="save-integration-card">
      <div className="save-integration-status"><span className="status-dot pending"/> Integração aguardando localização do projeto original</div>
      <h4>O que será preservado</h4>
      <ul>
        <li>Estrutura e formatos de backup já implementados.</li>
        <li>Regras de detecção e suporte aos jogos existentes.</li>
        <li>Histórico, restauração e validações já testadas.</li>
      </ul>
      <div className="save-integration-note"><ShieldCheck size={16}/> Nenhum backup será movido, convertido ou sobrescrito durante a integração.</div>
    </article>
    <div className="save-source-request"><Link2 size={17}/><span>Para conectar o código real, precisamos do repositório GitHub ou do ZIP do Save Manager. A interface não simula backups que ainda não estão conectados.</span></div>
  </section>;
}
