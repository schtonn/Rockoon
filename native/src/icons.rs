//! Small native vector icons, sharing the current theme and a consistent stroke.
use iced::{
    Alignment, Element,
    widget::{
        canvas::{self, Frame, Path, Stroke},
        row, text,
    },
};
#[derive(Clone, Copy, Debug)]
pub enum Icon {
    Sliders,
    Download,
    Globe,
    Map,
    Puzzle,
    Image,
    Folder,
    Search,
    Import,
    Plus,
    Refresh,
    Undo,
    Play,
    Eye,
    Check,
    Back,
    Left,
    Right,
    Down,
    External,
    Save,
    Trash,
    Edit,
    Close,
    More,
    Volume,
    Warning,
}
pub fn view<'a, M: 'a>(icon: Icon, size: f32) -> Element<'a, M> {
    iced::widget::canvas(Glyph(icon, None))
        .width(size)
        .height(size)
        .into()
}
pub fn label<'a, M: 'a>(icon: Icon, label: impl Into<String>) -> Element<'a, M> {
    row![view(icon, 16.0), text(label.into()).size(14)]
        .spacing(7)
        .align_y(Alignment::Center)
        .into()
}
pub fn label_color<'a, M: 'a>(
    icon: Icon,
    label: impl Into<String>,
    color: iced::Color,
) -> Element<'a, M> {
    row![
        iced::widget::canvas(Glyph(icon, Some(color)))
            .width(16)
            .height(16),
        text(label.into()).size(14)
    ]
    .spacing(7)
    .align_y(Alignment::Center)
    .into()
}
struct Glyph(Icon, Option<iced::Color>);
impl<M> canvas::Program<M> for Glyph {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        use iced::Point;
        let mut frame = Frame::new(renderer, bounds.size());
        let color = self.1.unwrap_or_else(|| crate::style::colors(theme).text);
        let s = bounds.width.min(bounds.height) / 24.0;
        let stroke = Stroke::default()
            .with_width(1.65 * s)
            .with_color(color)
            .with_line_cap(canvas::LineCap::Round)
            .with_line_join(canvas::LineJoin::Round);
        let point = |x: f32, y: f32| Point::new(x * s, y * s);
        let mut line = |coords: &[(f32, f32)], close: bool| {
            let path = Path::new(|p| {
                if let Some(&(x, y)) = coords.first() {
                    p.move_to(point(x, y));
                    for &(x, y) in &coords[1..] {
                        p.line_to(point(x, y));
                    }
                    if close {
                        p.close();
                    }
                }
            });
            frame.stroke(&path, stroke);
        };
        match self.0 {
            Icon::Folder => line(
                &[
                    (3., 6.),
                    (9., 6.),
                    (11., 9.),
                    (21., 9.),
                    (21., 20.),
                    (3., 20.),
                ],
                true,
            ),
            Icon::Download => {
                line(&[(12., 3.), (12., 16.)], false);
                line(&[(7., 11.), (12., 16.), (17., 11.)], false);
                line(&[(3., 16.), (3., 21.), (21., 21.), (21., 16.)], false);
            }
            Icon::Import => {
                line(&[(3., 15.), (3., 21.), (21., 21.), (21., 15.)], false);
                line(&[(12., 3.), (12., 15.)], false);
                line(&[(7., 8.), (12., 3.), (17., 8.)], false);
            }
            Icon::Plus => {
                line(&[(12., 5.), (12., 19.)], false);
                line(&[(5., 12.), (19., 12.)], false);
            }
            Icon::Check => line(&[(4., 12.), (9., 17.), (20., 6.)], false),
            Icon::Close => {
                line(&[(6., 6.), (18., 18.)], false);
                line(&[(6., 18.), (18., 6.)], false);
            }
            Icon::Left => line(&[(15., 5.), (8., 12.), (15., 19.)], false),
            Icon::Down => line(&[(5., 9.), (12., 16.), (19., 9.)], false),
            Icon::Right => line(&[(9., 5.), (16., 12.), (9., 19.)], false),
            Icon::Back => {
                line(&[(10., 5.), (3., 12.), (10., 19.)], false);
                line(&[(3., 12.), (21., 12.)], false);
            }
            Icon::External => {
                line(&[(14., 3.), (21., 3.), (21., 10.)], false);
                line(&[(21., 3.), (11., 13.)], false);
                line(
                    &[(10., 4.), (4., 4.), (4., 20.), (20., 20.), (20., 14.)],
                    false,
                );
            }
            Icon::Play => line(&[(7., 4.), (20., 12.), (7., 20.)], true),
            Icon::Map => {
                line(
                    &[
                        (3., 6.),
                        (9., 3.),
                        (15., 6.),
                        (21., 3.),
                        (21., 18.),
                        (15., 21.),
                        (9., 18.),
                        (3., 21.),
                    ],
                    true,
                );
                line(&[(9., 3.), (9., 18.)], false);
                line(&[(15., 6.), (15., 21.)], false);
            }
            Icon::Image => {
                line(&[(3., 3.), (21., 3.), (21., 21.), (3., 21.)], true);
                line(
                    &[(3., 16.), (9., 10.), (14., 15.), (17., 12.), (21., 16.)],
                    false,
                );
            }
            Icon::Sliders => {
                for (x, y) in [(5., 8.), (12., 16.), (19., 9.)] {
                    line(&[(x, 3.), (x, y - 2.)], false);
                    line(&[(x, y + 2.), (x, 21.)], false);
                    line(
                        &[
                            (x - 2., y - 2.),
                            (x + 2., y - 2.),
                            (x + 2., y + 2.),
                            (x - 2., y + 2.),
                        ],
                        true,
                    );
                }
            }
            Icon::Save => {
                line(
                    &[
                        (4., 3.),
                        (18., 3.),
                        (21., 6.),
                        (21., 21.),
                        (3., 21.),
                        (3., 3.),
                    ],
                    true,
                );
                line(&[(7., 3.), (7., 9.), (17., 9.), (17., 3.)], false);
                line(&[(7., 21.), (7., 14.), (17., 14.), (17., 21.)], false);
            }
            Icon::Trash => {
                line(&[(4., 6.), (20., 6.)], false);
                line(&[(9., 6.), (9., 3.), (15., 3.), (15., 6.)], false);
                line(&[(6., 6.), (7., 21.), (17., 21.), (18., 6.)], false);
                line(&[(10., 10.), (10., 17.)], false);
                line(&[(14., 10.), (14., 17.)], false);
            }
            Icon::Edit => {
                line(
                    &[(4., 16.), (15., 5.), (19., 9.), (8., 20.), (3., 21.)],
                    true,
                );
                line(&[(13., 7.), (17., 11.)], false);
            }
            Icon::Puzzle => line(
                &[
                    (3., 4.),
                    (8., 4.),
                    (8., 2.),
                    (12., 2.),
                    (12., 4.),
                    (20., 4.),
                    (20., 11.),
                    (22., 11.),
                    (22., 15.),
                    (20., 15.),
                    (20., 21.),
                    (13., 21.),
                    (13., 18.),
                    (9., 18.),
                    (9., 21.),
                    (3., 21.),
                    (3., 14.),
                    (6., 14.),
                    (6., 10.),
                    (3., 10.),
                ],
                true,
            ),
            Icon::Volume => {
                line(
                    &[
                        (3., 9.),
                        (7., 9.),
                        (12., 4.),
                        (12., 20.),
                        (7., 15.),
                        (3., 15.),
                    ],
                    true,
                );
                line(&[(16., 8.), (18., 10.), (18., 14.), (16., 16.)], false);
                line(&[(19., 4.), (22., 8.), (22., 16.), (19., 20.)], false);
            }
            Icon::Warning => {
                line(&[(12., 3.), (22., 21.), (2., 21.)], true);
                line(&[(12., 9.), (12., 14.)], false);
                line(&[(12., 17.), (12., 17.4)], false);
            }
            Icon::Undo => {
                line(&[(8., 4.), (3., 9.), (8., 14.)], false);
                line(
                    &[
                        (3., 9.),
                        (14., 9.),
                        (18., 11.),
                        (20., 15.),
                        (18., 19.),
                        (14., 20.),
                    ],
                    false,
                );
            }
            Icon::Globe | Icon::Search | Icon::Eye | Icon::Refresh | Icon::More => {}
        }
        match self.0 {
            Icon::More => {
                for y in [5.0, 12.0, 19.0] {
                    frame.fill(&Path::circle(point(12.0, y), 1.6 * s), color);
                }
            }
            Icon::Globe => {
                frame.stroke(&Path::circle(point(12., 12.), 9. * s), stroke);
                frame.stroke(
                    &Path::new(|p| {
                        p.move_to(point(12., 3.));
                        p.bezier_curve_to(point(6., 5.), point(6., 19.), point(12., 21.));
                        p.bezier_curve_to(point(18., 19.), point(18., 5.), point(12., 3.));
                    }),
                    stroke,
                );
                frame.stroke(&Path::line(point(3., 12.), point(21., 12.)), stroke);
            }
            Icon::Search => {
                frame.stroke(&Path::circle(point(10., 10.), 6.5 * s), stroke);
                frame.stroke(&Path::line(point(15., 15.), point(21., 21.)), stroke);
            }
            Icon::Eye => {
                let p = Path::new(|p| {
                    p.move_to(point(2., 12.));
                    p.bezier_curve_to(point(7., 3.), point(17., 3.), point(22., 12.));
                    p.bezier_curve_to(point(17., 21.), point(7., 21.), point(2., 12.));
                });
                frame.stroke(&p, stroke);
                frame.stroke(&Path::circle(point(12., 12.), 3. * s), stroke);
            }
            Icon::Refresh => {
                let p = Path::new(|p| {
                    p.move_to(point(19., 8.));
                    p.bezier_curve_to(point(13., -1.), point(1., 5.), point(4., 15.));
                    p.bezier_curve_to(point(6., 23.), point(18., 23.), point(21., 14.));
                });
                frame.stroke(&p, stroke);
                frame.stroke(
                    &Path::new(|p| {
                        p.move_to(point(20., 3.));
                        p.line_to(point(20., 9.));
                        p.line_to(point(14., 9.));
                    }),
                    stroke,
                );
            }
            _ => {}
        }
        vec![frame.into_geometry()]
    }
}

pub fn for_label(label: &str) -> Option<Icon> {
    Some(match label {
        "导入" => Icon::Import,
        "新建文件夹" => Icon::Plus,
        "刷新" | "重试" => Icon::Refresh,
        "游玩" => Icon::Play,
        "预览" => Icon::Eye,
        "打开文件夹" | "根目录" | "打开目录" | "更换目录" | "移动" => {
            Icon::Folder
        }
        "保存" | "保存配置" => Icon::Save,
        "删除" => Icon::Trash,
        "重命名" => Icon::Edit,
        "试听" => Icon::Volume,
        "配置" => Icon::Sliders,
        "撤销删除" => Icon::Back,
        "下载并安装" | "安装" | "安装支持" => Icon::Download,
        "浏览在线资源" => Icon::Globe,
        "社区资源站" => Icon::External,
        _ => return None,
    })
}
