# Demanda de UI — aproximar a aba Inputs do TRüsch e corrigir legibilidade no tema escuro

## Contexto

O FreeRusch (`apps/freerusch`) é um app egui com três abas: **Inputs**, **Results** e **Report**. A aba Inputs foge do padrão que um engenheiro de pontes reconhece, que é a tela do TRüsch 1.0, e no tema escuro o texto está apagado, com contraste ruim.

São duas entregas independentes: **organização da aba Inputs** e **legibilidade**. Faça as duas.

---

## Entrega 1 — organização da aba Inputs

Referência de organização (TRüsch 1.0), de cima para baixo. Reproduza a **estrutura**, não os pixels.

### 1. Barra de menus
Ordem e nomes: **Arquivo**, **Opções**, **Resultados**, **Sobre**. A barra de menus do egui já existe; apenas organize os itens assim.

### 2. Linha superior — dois painéis lado a lado

**Esquerda (~40% da largura): desenho esquemático.**
- Laje em corte: retângulo representando a seção, com as bordas desenhadas conforme a vinculação escolhida (apoio = triângulo, engaste = hachura, livre = traço solto, indefinida = tracejado).
- Cota do vão `lx` na base, cota da espessura `t`, cota da largura de distribuição `a`.
- Área de carga em trapézio sobre a faixa de tráfego, com o valor do momento em exibição escrito sobre ele.
- **O desenho reage aos campos**: mudou `lx`, `t`, a vinculação ou a direção do tráfego, o desenho muda.
- Tudo é `egui::Painter` (vetor). Nenhuma imagem, nenhum bitmap, nada copiado do TRüsch.

**Direita: parâmetros de entrada.**
- **Dimensões:** `lx` (editável), `ly` (mostrar `∞`, desabilitado), `t` (editável), `a` (calculado, somente leitura, visualmente distinto dos editáveis).
- **Vinculações:** quatro seletores, um por borda (superior, inferior, esquerda, direita), com as opções *Indefinida / Apoiada / Engastada / Livre*.
- **Direção do tráfego:** dois botões de rádio com seta — vertical (direção y) e horizontal (direção x).
- **Coeficiente de impacto φ:** campo numérico, com o valor vindo do motor (não recalcular na UI).

### 3. Barra de ações
Entre os parâmetros e a tabela: botão **Calcular** e três alternadores — **Mxm**, **Mym**, **Mxe** — que escolhem qual esforço a tabela e o painel de resultados mostram. Um deles sempre ativo.

### 4. Tabela de coeficientes (centro)
- Cabeçalho em dois níveis: grupos por `t/a` (0,125 · 0,250 · 0,5 · 1,0), cada grupo com a subcoluna `L`; e um grupo "Para todos os valores de t/a" com as subcolunas `p` e `p′`.
- Primeira coluna: `lx/a`, de 0,5 a 10,0.
- **Destacar a linha e a célula usadas na interpolação atual** (linha de `lx/a` e coluna de `t/a`), nos dois temas.

### 5. Painel de resultados (direita)
- **Tabela utilizada:** o número da tabela escolhida.
- **Cargas móveis:** `ML`, `Mp`, `Mp′` e o momento resultante, com unidade `kN·m/m`.
- **Cargas permanentes:** o coeficiente `k` e o momento resultante.
- Todos os valores vêm de `freerusch-core`. A UI não calcula nada.

### 6. Rodapé
Barra de status curta com a unidade em uso e o estado do cálculo (por exemplo *Calculado* / *Alterado após o último cálculo*). O TRüsch não tem rodapé; isso é acréscimo deliberado, para o usuário saber se o resultado está atualizado.

---

## Entrega 2 — legibilidade no tema escuro

- Texto de corpo com contraste **≥ 7:1** sobre o fundo do painel; texto secundário **≥ 4,5:1**.
- Aumentar o tamanho base: sugestão 14 pt para corpo e 16 pt para títulos de painel, com respiro entre linhas.
- Números da tabela em fonte com dígitos tabulares, para alinhar as casas decimais entre linhas.
- Não usar cinza fino nem `Color32::from_gray(...)`: apenas `theme::pal().TEXT` e `theme::pal().TEXT_WEAK`.
- Linhas de grade visíveis, porém discretas.
- O realce da linha/célula em uso precisa funcionar em claro **e** escuro — o azul do TRüsch não tem contraste suficiente no fundo preto; derive a cor do tema.

---

## Restrições

- `egui` + `craft-ui` apenas; nenhuma dependência nova.
- Nenhuma cor fixa: só `theme::pal()`.
- Nenhum `unwrap`/`expect` em código de UI; trate ausências.
- Não alterar o motor de cálculo, nem o formato `.frus`, nem a assinatura pública de `freerusch-core`.
- `cargo fmt --check` e `cargo clippy -D warnings` limpos; os 38 testes atuais continuam verdes.
- Nenhum valor normativo inventado: `φ`, `a`, coeficientes e tabela vêm do core.

---

## Critério de aceite

1. Aba Inputs com a organização descrita, em janela de 1440×900, sem rolagem horizontal.
2. Tema escuro e claro conferidos visualmente; textos legíveis nos dois.
3. `cargo test --workspace` verde e `clippy` limpo.
4. Testes de validação do workbook continuam passando (nenhuma regressão de número).
5. Um screenshot da aba Inputs no tema escuro, salvo no repositório em `docs/img/inputs-dark.png` ou anexado ao relatório final.

Se algum item da referência não fizer sentido no FreeRusch (por exemplo, `ly = ∞` quando o motor só trata um caso), **não invente**: implemente o que fizer sentido e registre a diferença na resposta.
